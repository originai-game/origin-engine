use super::*;

impl Lockstep {
    pub(super) fn message(&mut self, from: u32, d: &Value) {
        if self.handover.is_some() && d["k"] != "start" {
            self.deferred.push((from, d.clone()));
            return;
        }
        match d["k"].as_str().unwrap_or_default() {
            "start" => {
                if self.run.is_some() {
                    return;
                }
                match parse_start(d) {
                    Ok((run, tick, snapshot, inputs, last)) => {
                        for (t, s, b) in inputs {
                            self.inputs.entry(t).or_default().insert(s, b);
                        }
                        for (s, t) in last {
                            let e = self.seat_last.entry(s).or_insert(t);
                            *e = (*e).max(t);
                        }
                        self.handover = Some((run, tick, snapshot));
                    }
                    Err(why) => self
                        .notes
                        .push(format!("lockstep: peer {from}'s start was refused: {why}")),
                }
            }
            "admit" => {
                let (Some(peer), Some(tick), Some(since)) =
                    (u32_of(&d["peer"]), u64_of(&d["tick"]), u64_of(&d["since"]))
                else {
                    return;
                };
                if Some(peer) == self.me {
                    return;
                }
                let Some(run) = self.run.as_mut() else { return };
                if !run.admitted.insert(peer) {
                    return;
                }
                run.running.insert(peer);
                if let Some(seat) = u32_of(&d["seat"]).filter(|s| *s < run.players) {
                    run.seats[seat as usize].push(Hold {
                        peer,
                        since,
                        until: None,
                        left: false,
                    });
                    // A tick from `since` this peer predicted ran without
                    // the seat's new holder (and its og.SeatRejoined).
                    if self.frontier > since {
                        self.mark_wrong(since);
                    }
                }
                self.heard.insert(peer, self.now);
                // The newcomer has none of this peer's inputs from its
                // snapshot's tick on.
                self.acks.insert(peer, tick.saturating_sub(1));
                self.send_inputs(Some(peer));
            }
            "inputs" => {
                if let Some(a) = u64_of(&d["a"])
                    && self.run.as_ref().is_some_and(|r| r.running.contains(&from))
                {
                    let e = self.acks.entry(from).or_insert(a);
                    *e = (*e).max(a);
                }
                let (Some(t0), Some(s), Some(since)) =
                    (u64_of(&d["t"]), u32_of(&d["s"]), u64_of(&d["h"]))
                else {
                    return;
                };
                // Sent after a seat change this peer has not heard yet (an
                // unreliable message overtakes a reliable one): its inputs
                // may confirm a tick the change moves, so they wait for
                // the resend.
                if self
                    .run
                    .as_ref()
                    .is_none_or(|r| u64_of(&d["e"]).is_some_and(|e| e > r.epoch()))
                {
                    return;
                }
                let list = d["i"].as_array().cloned().unwrap_or_default();
                for (i, b) in list.iter().enumerate() {
                    let t = t0 + i as u64;
                    // Only the seat's holder at t, in the hold it sent for:
                    // never a dropped peer's inputs from before its drop.
                    let holds = self.run.as_ref().and_then(|r| {
                        r.seats
                            .get(s as usize)?
                            .iter()
                            .find(|h| h.covers(t))
                            .map(|h| h.peer == from && h.since == since)
                    });
                    if holds != Some(true) {
                        continue;
                    }
                    let bytes = b.as_str().and_then(base64::decode).unwrap_or_default();
                    self.take_input(t, s, bytes);
                }
            }
            "drop?" => self.drop_asked(from, d),
            "drop!" => {
                let (Some(s), Some(since), Some(last), Some(t0)) = (
                    u32_of(&d["s"]),
                    u64_of(&d["since"]),
                    u64_of(&d["last"]),
                    u64_of(&d["t"]),
                ) else {
                    return;
                };
                let Some(p) = self.proposals.get_mut(&s).filter(|p| p.since == since) else {
                    return;
                };
                if !p.waiting.remove(&from) {
                    return;
                }
                p.last = p.last.max(last);
                p.low = p.low.min(last);
                for (i, b) in d["i"].as_array().into_iter().flatten().enumerate() {
                    let t = t0 + i as u64;
                    if t > p.from
                        && t <= last
                        && let Some(bytes) = b.as_str().and_then(base64::decode)
                    {
                        p.inputs.entry(t).or_insert(bytes);
                    }
                }
            }
            "drop" => {
                // From a running peer, or a lower one: after a partition the
                // higher side gives way and rejoins.
                let trusted = self.run.as_ref().is_some_and(|r| r.running.contains(&from))
                    || self.me.is_some_and(|me| from < me);
                if trusted {
                    self.apply_drop(d);
                }
            }
            "back" => {}
            "hash" => {
                let Some(first) = u64_of(&d["from"]) else {
                    return;
                };
                let list = d["h"].as_array().cloned().unwrap_or_default();
                let theirs = self.theirs.entry(from).or_default();
                for (i, h) in list.iter().enumerate() {
                    if let Some(h) = unhex(h) {
                        theirs.insert(first + i as u64, h);
                    }
                }
                self.compare(from);
            }
            "snap?" => {
                let Some(t) = u64_of(&d["t"]) else { return };
                let answer = match self.mine.get(&t) {
                    Some((_, s)) => json!({ "k": "snap", "t": t, "s": base64::encode(s) }),
                    None => json!({ "k": "snap", "t": t, "missing": true }),
                };
                self.send(Some(from), answer);
            }
            "snap" => {
                let Some(t) = u64_of(&d["t"]) else { return };
                let Some(ds) = self
                    .desync
                    .as_mut()
                    .filter(|x| x.peer == from && x.tick == t)
                else {
                    return;
                };
                match d["s"].as_str().and_then(base64::decode) {
                    Some(bytes) => {
                        ds.diff = Some(
                            match (Snapshot::decode(&ds.local), Snapshot::decode(&bytes)) {
                                (Ok(a), Ok(b)) => diff(&a, &b),
                                (Err(e), _) | (_, Err(e)) => json!({ "error": e }),
                            },
                        );
                        ds.remote = Some(bytes);
                    }
                    None => {
                        ds.diff = Some(
                            json!({ "error": format!("peer {from} no longer kept its snapshot of tick {t}") }),
                        );
                    }
                }
            }
            _ => {}
        }
    }

    /// Compares what both sides hashed with `peer`, in tick order.
    pub(super) fn compare(&mut self, peer: u32) {
        let Some(theirs) = self.theirs.get_mut(&peer) else {
            return;
        };
        let ticks: Vec<u64> = theirs
            .keys()
            .copied()
            .filter(|t| self.mine.contains_key(t))
            .collect();
        for t in ticks {
            let h = theirs.remove(&t).expect("listed");
            let (mine, snapshot) = &self.mine[&t];
            if *mine == h {
                let c = self.compared.entry(peer).or_insert(t);
                *c = (*c).max(t);
                continue;
            }
            if self.desync.is_none() {
                self.desync = Some(Desync {
                    tick: t,
                    peer,
                    local_hash: *mine,
                    remote_hash: h,
                    local: snapshot.clone(),
                    remote: None,
                    diff: None,
                });
                self.desync_event = Some((t, peer, 1));
                self.notes.push(format!(
                    "lockstep: DESYNC at tick {t}: this world hashes {}, peer {peer}'s {}; dump_desync names what differs",
                    hex(*mine),
                    hex(h)
                ));
                self.out
                    .push((Some(peer), json!({ "k": "snap?", "t": t }), false));
            }
            // After the first divergence everything later differs too.
            theirs.clear();
            break;
        }
    }

    /// Whether every seat occupied at `tick` has its real input for it.
    fn known(&self, tick: u64) -> bool {
        let Some(run) = &self.run else { return true };
        let have = self.inputs.get(&tick);
        (0..run.seats.len()).all(|s| {
            run.hold(s, tick).is_none() || have.is_some_and(|m| m.contains_key(&(s as u32)))
        })
    }

    pub(super) fn advance_confirmed(&mut self) {
        let limit = self.frontier + CONFIRM_AHEAD;
        while self.confirmed < limit && self.known(self.confirmed + 1) {
            self.confirmed += 1;
        }
    }

    /// The inputs tick `tick` runs with: the real ones, else each seat's
    /// last real input from its holder (empty before the holder's first).
    pub(super) fn inputs_for(&self, tick: u64) -> Vec<Vec<u8>> {
        let run = self.run.as_ref().expect("running");
        let have = self.inputs.get(&tick);
        (0..run.seats.len())
            .map(|s| {
                let seat = s as u32;
                let Some(hold) = run.hold(s, tick) else {
                    return Vec::new();
                };
                if let Some(b) = have.and_then(|m| m.get(&seat)) {
                    return b.clone();
                }
                // A prediction repeats the last state, never its commands.
                let mut b = self
                    .inputs
                    .range(hold.since..tick)
                    .rev()
                    .find_map(|(_, m)| m.get(&seat).cloned())
                    .unwrap_or_default();
                if self.state_bytes > 0 {
                    b.truncate(self.state_bytes);
                }
                b
            })
            .collect()
    }
}
