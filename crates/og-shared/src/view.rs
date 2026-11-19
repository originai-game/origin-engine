//! A client's side of the protocol: the remote avatars it was sent, the
//! snapshots it received (its acknowledgements), and its own proposals'
//! baselines. The native client, the engine's Worker transport and the load
//! test's bots all decode through it.

use crate::protocol::{
    ACK_WINDOW, AvatarState, ClientMsg, Enter, Propose, SnapshotHead, Update, mask,
};
use crate::wire::{Reader, WireError, WireResult, Writer};
use std::collections::{HashMap, VecDeque};

/// Whether `t` is in the set a `(newest, bits)` acknowledgement names: bit
/// i is tick `newest - 1 - i`.
pub fn acked(newest: u32, bits: u32, t: u32) -> bool {
    if t == newest {
        return true;
    }
    if t > newest {
        return false;
    }
    let back = newest - 1 - t;
    back < ACK_WINDOW && bits >> back & 1 == 1
}

/// One remote avatar this client holds.
#[derive(Clone, Debug)]
pub struct Remote {
    pub player: String,
    pub name: String,
    pub kind: u32,
    pub enter_tick: u32,
    /// Its newest state.
    pub state: AvatarState,
    /// The tick of its newest state.
    pub tick: u32,
    /// The tick of the newest update that said it jumped.
    pub jumped: Option<u32>,
    /// Its states in received snapshots, for baselines.
    history: VecDeque<(u32, AvatarState)>,
}

/// What a snapshot changed.
#[derive(Clone, Debug, PartialEq)]
pub struct Applied {
    pub head: SnapshotHead,
    pub updates: Vec<Update>,
}

#[derive(Default)]
pub struct View {
    pub remotes: HashMap<u16, Remote>,
    newest: Option<u32>,
    bits: u32,
    /// Snapshots dropped for arriving after a newer one.
    pub stale: u64,
    /// The newest proposal the server has, from the newest snapshot.
    pub ack_seq: u32,
    /// Avatars in sight but not sent, from the newest snapshot.
    pub others: u32,
}

/// A snapshot that updates an avatar whose enter has not arrived yet. Over
/// WebTransport the snapshot is a datagram and the enter is on the stream,
/// so a stream packet lost and resent lets later snapshots overtake it.
/// The snapshot is rejected whole and not acknowledged, as any that does
/// not apply, so the server never builds on it; the next one after the
/// enter applies. Expected under loss, not a fault: callers count it apart
/// from messages that do not decode.
pub const NOT_ENTERED: WireError = WireError("an update for an avatar not entered");

impl View {
    pub fn new() -> View {
        View::default()
    }

    /// The acknowledgement to send: the newest snapshot tick and the bits below it.
    pub fn ack(&self) -> (u32, u32) {
        (self.newest.unwrap_or(0), self.bits)
    }

    pub fn enter(&mut self, e: &Enter) {
        self.remotes.insert(
            e.id,
            Remote {
                player: e.player.clone(),
                name: e.name.clone(),
                kind: e.kind,
                enter_tick: e.tick,
                state: e.state,
                tick: e.tick,
                jumped: None,
                history: VecDeque::new(),
            },
        );
    }

    pub fn leave(&mut self, id: u16) -> Option<Remote> {
        self.remotes.remove(&id)
    }

    /// Decode and apply a snapshot. `Ok(None)`: it is older than one
    /// already applied and is dropped (its tick is never acknowledged, so
    /// the server never builds on it). A snapshot that does not decode
    /// changes nothing and is not acknowledged either.
    pub fn snapshot(&mut self, bytes: &[u8]) -> WireResult<Option<Applied>> {
        let mut r = Reader::new(bytes);
        let (head, count) = SnapshotHead::read(&mut r)?;
        if self.newest.is_some_and(|n| head.tick <= n) {
            self.stale += 1;
            return Ok(None);
        }
        let mut updates = Vec::with_capacity(count);
        for _ in 0..count {
            let id =
                u16::try_from(r.varint()?).map_err(|_| WireError("an avatar id past 16 bits"))?;
            let m = r.u8()?;
            let remote = self.remotes.get_mut(&id).ok_or(NOT_ENTERED)?;
            let base = match head.base {
                _ if m & mask::WHOLE != 0 => None,
                None => None,
                Some(b) => {
                    let ticks = remote.history.iter().map(|(t, _)| *t);
                    let i = baseline_index(ticks, remote.enter_tick, b, |_| true)
                        .ok_or(WireError("an update's baseline is not held"))?;
                    remote.history.drain(..i);
                    Some(remote.history[0].1)
                }
            };
            let state = AvatarState::read_parts(m & mask::STATE, base.as_ref(), &mut r)?;
            updates.push(Update {
                id,
                jump: m & mask::JUMP != 0,
                state,
            });
        }
        if !r.done() {
            return Err(WireError("bytes after the snapshot"));
        }
        // Commit.
        self.bits = match self.newest {
            None => 0,
            Some(n) => {
                let shift = u64::from(head.tick - n);
                if shift > u64::from(ACK_WINDOW) {
                    0
                } else {
                    (((u64::from(self.bits) << shift) | (1u64 << (shift - 1))) & 0xffff_ffff) as u32
                }
            }
        };
        self.newest = Some(head.tick);
        self.ack_seq = head.ack_seq;
        self.others = head.others;
        let floor = head.tick.saturating_sub(2 * ACK_WINDOW);
        for u in &updates {
            let remote = self.remotes.get_mut(&u.id).expect("checked above");
            remote.state = u.state;
            remote.tick = head.tick;
            if u.jump {
                remote.jumped = Some(head.tick);
            }
            remote.history.push_back((head.tick, u.state));
        }
        for remote in self.remotes.values_mut() {
            while remote.history.front().is_some_and(|(t, _)| *t < floor) {
                remote.history.pop_front();
            }
        }
        Ok(Some(Applied { head, updates }))
    }
}

/// Which entry of a history (its ticks, oldest first) is an avatar's
/// baseline for a snapshot built on acknowledged tick `base`: the newest at
/// or before `base`, within the window, in the set `received` names, and
/// not older than its enter. The server and the client apply this one rule
/// to the same set, so they agree.
///
/// The entries older than it can go: the server builds each snapshot on
/// its newest acknowledgement, which only moves forward, and a client
/// applies snapshots in tick order, so no later snapshot's baseline is
/// older. A history thus holds about a round trip of entries.
pub fn baseline_index(
    ticks: impl DoubleEndedIterator<Item = u32> + ExactSizeIterator,
    enter_tick: u32,
    base: u32,
    received: impl Fn(u32) -> bool,
) -> Option<usize> {
    ticks
        .enumerate()
        .rev()
        .filter(|(_, t)| *t <= base)
        .take_while(|(_, t)| base - *t <= ACK_WINDOW && *t >= enter_tick)
        .find(|(_, t)| received(*t))
        .map(|(i, _)| i)
}

/// A client's own proposals: their numbers and the states they carried,
/// so each can be a delta over one the server has.
#[derive(Default)]
pub struct Proposer {
    pub seq: u32,
    sent: VecDeque<(u32, AvatarState)>,
}

impl Proposer {
    /// Encode the next proposal of `state`, over the newest one the server
    /// has (`server_has`, a snapshot's `ack_seq`) while this still holds it.
    pub fn propose(
        &mut self,
        state: AvatarState,
        teleport: bool,
        ack: (u32, u32),
        server_has: u32,
    ) -> Vec<u8> {
        self.seq += 1;
        let base = self
            .sent
            .iter()
            .find(|(s, _)| *s == server_has && self.seq - *s < ACK_WINDOW)
            .map(|(s, st)| (*s, *st));
        let p = Propose {
            seq: self.seq,
            base: base.map_or(0, |(s, _)| s),
            ack_tick: ack.0,
            ack_bits: ack.1,
            teleport,
            state,
            sections: Vec::new(),
        };
        let mut w = Writer::new();
        ClientMsg::encode_propose(&p, base.as_ref().map(|(_, st)| st), &mut w);
        self.sent.push_back((self.seq, state));
        while self.sent.len() > ACK_WINDOW as usize {
            self.sent.pop_front();
        }
        w.buf
    }

    /// The state proposal `seq` carried.
    pub fn state(&self, seq: u32) -> Option<AvatarState> {
        self.sent.iter().find(|(s, _)| *s == seq).map(|(_, st)| *st)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn acknowledgement_bits_name_exactly_the_ticks_received() {
        let mut v = View::new();
        let snap = |tick: u32| {
            let mut w = Writer::new();
            SnapshotHead {
                tick,
                base: None,
                ack_seq: 0,
                others: 0,
            }
            .write(0, &mut w);
            w.buf
        };
        for t in [10, 11, 13, 20] {
            v.snapshot(&snap(t)).unwrap().unwrap();
        }
        // 12 never came; a late 12 after 20 is dropped, not acknowledged.
        assert!(v.snapshot(&snap(12)).unwrap().is_none());
        let (n, bits) = v.ack();
        assert_eq!(n, 20);
        let got: Vec<u32> = (0..=20).filter(|&t| acked(n, bits, t)).collect();
        assert_eq!(got, vec![10, 11, 13, 20]);
        // A gap past the window forgets everything older.
        v.snapshot(&snap(60)).unwrap().unwrap();
        let (n, bits) = v.ack();
        assert_eq!((0..=60).filter(|&t| acked(n, bits, t)).count(), 1);
    }
}
