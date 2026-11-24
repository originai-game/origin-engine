//! One world server's state (docs/design.md §2.12), with no IO: the server
//! hands it what each connection said and takes what to send each one.
//!
//! Each tick ([`Shard::step`], `tickHz` a second) it lets lingering avatars
//! go, rebuilds the interest grid, and for every connected client:
//!
//! - every [`REFRESH_TICKS`] ticks (staggered by session), picks its set:
//!   the avatars of its world within `radius` cells of its own (on x and
//!   z), and those already in the set within `radius + 1` (hysteresis),
//!   nearest first, at most `maxVisible` (members count as a little nearer,
//!   so the set does not churn at its edge); on the other ticks only a
//!   member whose avatar went leaves;
//! - sends `leave` for each that left the set and `enter`, whole, for each
//!   that came in;
//! - raises each member's priority by its rank's weight (the nearest 16 by
//!   1, the next 24 by ½, the rest by ¼: 20, 10 and 5 updates a second at
//!   20 Hz) and writes, highest first, those at 1 or more into the
//!   snapshot until its share of `bytesPerSecond` is spent (Tribes 2's
//!   accumulator); one that did not fit keeps its priority and goes first
//!   next time;
//! - writes each as a delta over its baseline (protocol.rs's rule), and
//!   leaves out one with nothing new.
//!
//! A proposal is checked before it is taken: inside the bounds, a velocity
//! the kind may have, the game's field bytes the hello named, and a move
//! no longer than the avatar's movement credit, which grows by `maxSpeed`
//! a second up to half a second's worth (so proposals that arrive bunched
//! pass, and a steady speed past `maxSpeed` does not). A teleport skips the
//! credit when the teleport policy allows. A refused proposal is not taken,
//! and the client hears a `correct` with the last state taken (at most four
//! a second).

use crate::config::{Config, TeleportPolicy};
use crate::protocol::{
    ACK_WINDOW, Authority, AvatarState, ClientMsg, Enter, Hello, MAX_EVENT, MAX_FIELDS, ObjectOp,
    PROTOCOL, Refusal, ServerMsg, SharedObject, SnapshotHead, Welcome, mask,
};
use crate::view::{acked, baseline_index};
use crate::wire::Writer;
use std::collections::{BTreeMap, BTreeSet, HashMap, VecDeque};
use std::hash::{BuildHasherDefault, Hasher};

/// FxHash's mix (rustc's hasher): keys here are random session ids and small
/// integers, never chosen by a client, so a fast unkeyed hash is enough.
#[derive(Default, Clone, Copy)]
struct Mix(u64);

impl Hasher for Mix {
    fn finish(&self) -> u64 {
        self.0
    }
    fn write(&mut self, bytes: &[u8]) {
        for b in bytes {
            self.write_u64(u64::from(*b));
        }
    }
    fn write_u64(&mut self, v: u64) {
        self.0 = (self.0.rotate_left(5) ^ v).wrapping_mul(0x517c_c1b7_2722_0a95);
    }
    fn write_u32(&mut self, v: u32) {
        self.write_u64(u64::from(v));
    }
    fn write_i32(&mut self, v: i32) {
        self.write_u64(u64::from(v as u32));
    }
}

type Fast<K, V> = HashMap<K, V, BuildHasherDefault<Mix>>;

/// Who a verified token says the client is.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Claims {
    /// The player's opaque, game-scoped id.
    pub player: String,
    /// The display name; the player id when empty.
    pub name: String,
    /// The avatar kind the token fixes, over the hello's.
    pub kind: Option<u32>,
    /// The only world the token is good for.
    pub world: Option<String>,
}

/// Counters for metrics and the load test.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Stats {
    pub joins: u64,
    pub refusals: u64,
    pub proposals: u64,
    pub rejected: u64,
    pub corrections: u64,
    pub events: u64,
    pub enters: u64,
    pub leaves: u64,
    pub updates: u64,
    pub ops: u64,
    pub ops_refused: u64,
}

/// A change to the shared objects, for a store to keep ([`Shard::take_changes`]).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Change {
    Put { world: String, object: SharedObject },
    Remove { world: String, key: String },
}

/// One world's shared objects, by key, and the keys in each cell (x, z).
#[derive(Default)]
struct Objects {
    by_key: BTreeMap<String, SharedObject>,
    cells: HashMap<(i32, i32), BTreeSet<String>>,
}

impl Objects {
    fn put(&mut self, o: SharedObject) {
        if let Some(old) = self.by_key.get(&o.key) {
            let c = (old.place.cell[0], old.place.cell[2]);
            if let Some(set) = self.cells.get_mut(&c) {
                set.remove(&o.key);
            }
        }
        self.cells
            .entry((o.place.cell[0], o.place.cell[2]))
            .or_default()
            .insert(o.key.clone());
        self.by_key.insert(o.key.clone(), o);
    }

    fn remove(&mut self, key: &str) -> Option<SharedObject> {
        let o = self.by_key.remove(key)?;
        if let Some(set) = self.cells.get_mut(&(o.place.cell[0], o.place.cell[2])) {
            set.remove(key);
        }
        Some(o)
    }
}

fn near(a: [i32; 3], b: [i32; 3], r: i32) -> bool {
    (a[0] - b[0]).abs() <= r && (a[2] - b[2]).abs() <= r
}

struct Avatar {
    world: u32,
    /// The world's name (shared objects are kept by it, past its last member).
    world_name: String,
    player: String,
    name: String,
    kind: u32,
    fields_len: u8,
    state: Option<AvatarState>,
    /// Counts the jumps (the first state, teleports), so each viewer marks one once.
    jumps: u32,
    /// The first tick whose state has its current fields.
    fields_at: u32,
    credit: f32,
    last_seq: u32,
    received: VecDeque<(u32, AvatarState)>,
    events: f32,
    last_correct: Option<u32>,
    /// Its place in the tick's dense state ([`Cur`]).
    slot: u32,
}

/// An avatar's state this tick, by slot: what a client's update reads, kept
/// small and dense so the 1.3 million updates a second at 2000 clients read
/// one cache-friendly array instead of a map of whole avatars.
#[derive(Clone, Copy, Default)]
struct Cur {
    jumps: u32,
    fields_at: u32,
    state: Option<AvatarState>,
}

/// How many of its last send ticks the server keeps of an avatar for a
/// client: acknowledgements lag a round trip, a few ticks; past these an
/// update goes `WHOLE`.
const SENT: usize = 4;

/// Ticks of an avatar's state the shard keeps, per slot: a baseline is at
/// most `ACK_WINDOW` older than an acknowledgement that is itself at most
/// `ACK_WINDOW` old.
const RING: usize = 2 * ACK_WINDOW as usize + 2;

/// The ticks a client was last sent one avatar at, oldest first. The states
/// they carried are the avatar's at those ticks, in the shard's ring: one
/// copy per avatar instead of one per client (per-client copies were 32 MB
/// walked every tick at 2000 clients). A tick is kept as its low 16 bits:
/// none is older than `2 * ACK_WINDOW` plus a refresh (dropped on the set's
/// refresh), so the tick it names is the latest one with those bits.
#[derive(Clone, Copy)]
struct Sent {
    len: u8,
    ticks: [u16; SENT],
}

impl Sent {
    fn new() -> Sent {
        Sent {
            len: 0,
            ticks: [0; SENT],
        }
    }

    /// The ticks, as of `now`.
    fn ticks(&self, now: u32) -> impl DoubleEndedIterator<Item = u32> + ExactSizeIterator + '_ {
        self.ticks[..usize::from(self.len)]
            .iter()
            .map(move |&t| now.wrapping_sub(u32::from((now as u16).wrapping_sub(t))))
    }

    /// Drop the first `n` entries.
    fn drop_front(&mut self, n: usize) {
        let len = usize::from(self.len);
        self.ticks.copy_within(n..len, 0);
        self.len = (len - n) as u8;
    }

    fn push(&mut self, tick: u32) {
        if usize::from(self.len) == SENT {
            self.drop_front(1);
        }
        self.ticks[usize::from(self.len)] = tick as u16;
        self.len += 1;
    }
}

/// An avatar in a client's set.
#[derive(Clone, Copy)]
struct Seen {
    id: u16,
    /// Its place by distance when the set was last chosen.
    rank: u16,
    enter_tick: u32,
    sent: Sent,
    /// The avatar's jump count when last sent, low bits: compared for a change only.
    jumps: u16,
    priority: f32,
    /// Its avatar's slot and the generation that held it when it entered.
    slot: u32,
    generation: u32,
}

struct Viewer {
    connected: bool,
    lost_at: u32,
    ack: Option<(u32, u32)>,
    /// Its set, nearest first as of the last choice: a small dense array
    /// (a map of 64 entries per client was 16 MB walked every tick at 2000
    /// clients); avatars are matched to it by slot and generation.
    set: Vec<Seen>,
    next_id: u16,
    out: Vec<Vec<u8>>,
    /// Avatars within the radius not in the set, as of the last tick.
    others: u32,
    /// The most bytes one snapshot may take: its transport's datagram
    /// (WebTransport), else unbounded but for the byte budget.
    cap: usize,
    /// The shared objects it holds, by key, and the version it was sent.
    objects: HashMap<String, u32>,
}

struct WorldShape {
    shape: u64,
    fields_len: u8,
    members: usize,
}

pub struct Shard {
    pub config: Config,
    pub tick: u32,
    pub stats: Stats,
    draining: bool,
    avatars: Fast<u64, Avatar>,
    viewers: Fast<u64, Viewer>,
    worlds: HashMap<String, (u32, WorldShape)>,
    next_world: u32,
    grid: Grid,
    session_seed: u64,
    session_count: u64,
    /// Sessions the server should close (replaced by a newer join of their player).
    closed: Vec<u64>,
    /// Each slot's generation (bumped when it is given out) and the free ones.
    slot_gens: Vec<u32>,
    free_slots: Vec<u32>,
    cur: Vec<Cur>,
    /// By slot, what every client checks of every member each tick, apart
    /// so it stays in cache: the generation of the avatar holding it if it
    /// has a state (0: none) and its world.
    alive: Vec<(u32, u32)>,
    /// The last `RING` ticks' states by slot, at `[tick % RING][slot]`: a
    /// row per tick, so the baselines a tick reads (a few ticks back) are a
    /// few contiguous rows, not a cache miss each.
    ring: Vec<Vec<Motion>>,
    /// Shared objects by world name; they outlive their world's members.
    objects: HashMap<String, Objects>,
    /// Changes to them since the last [`Shard::take_changes`].
    changes: Vec<Change>,
}

/// What a baseline needs of a past state: its motion. Whether its fields
/// differ from now is [`Cur::fields_at`] against its tick, so the ring
/// holds 28 bytes a state instead of 60 (the read every update makes).
#[derive(Clone, Copy, Default)]
struct Motion {
    cell: [i32; 3],
    pos: [u16; 3],
    vel: [i16; 3],
    yaw: u16,
}

impl Motion {
    fn of(s: &AvatarState) -> Motion {
        Motion {
            cell: s.cell,
            pos: s.pos,
            vel: s.vel,
            yaw: s.yaw,
        }
    }

    /// The state with this motion and `now`'s fields.
    fn with_fields_of(self, now: &AvatarState) -> AvatarState {
        AvatarState {
            cell: self.cell,
            pos: self.pos,
            vel: self.vel,
            yaw: self.yaw,
            ..*now
        }
    }
}

/// The avatars in each cell of each world: session, slot and position.
type Grid = Fast<(u32, i32, i32), Vec<(u64, u32, [f64; 3])>>;

/// How often a client's set is chosen again, in ticks (5 Hz at 20 Hz).
pub const REFRESH_TICKS: u64 = 4;

/// The rank weights of the rate tiers.
fn weight(rank: usize) -> f32 {
    match rank {
        0..16 => 1.0,
        16..40 => 0.5,
        _ => 0.25,
    }
}

fn splitmix(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9e37_79b9_7f4a_7c15);
    x = (x ^ (x >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    x ^ (x >> 31)
}

fn dist2(a: [f64; 3], b: [f64; 3]) -> f64 {
    (0..3).map(|i| (a[i] - b[i]).powi(2)).sum()
}

impl Shard {
    /// A shard; `seed` makes its session ids hard to guess.
    pub fn new(config: Config, seed: u64) -> Shard {
        Shard {
            config,
            tick: 1,
            stats: Stats::default(),
            draining: false,
            avatars: Fast::default(),
            viewers: Fast::default(),
            worlds: HashMap::new(),
            next_world: 0,
            grid: Fast::default(),
            session_seed: seed,
            session_count: 0,
            closed: Vec::new(),
            slot_gens: Vec::new(),
            free_slots: Vec::new(),
            cur: Vec::new(),
            alive: Vec::new(),
            ring: vec![Vec::new(); RING],
            objects: HashMap::new(),
            changes: Vec::new(),
        }
    }

    /// Clients present, connected or lingering.
    pub fn players(&self) -> usize {
        self.avatars.len()
    }

    pub fn connected(&self) -> usize {
        self.viewers.values().filter(|v| v.connected).count()
    }

    pub fn draining(&self) -> bool {
        self.draining
    }

    /// Refuse new clients from now on and tell each connected one to move.
    pub fn drain(&mut self, reason: &str) {
        self.draining = true;
        let moving = ServerMsg::Moving {
            reason: reason.into(),
        }
        .to_bytes();
        for v in self.viewers.values_mut().filter(|v| v.connected) {
            v.out.push(moving.clone());
        }
    }

    /// Take a client in after its token was verified. The welcome is
    /// already queued; the error is the refusal to send.
    pub fn hello(&mut self, h: &Hello, c: Claims) -> Result<u64, (Refusal, String)> {
        let refused = |this: &mut Shard, r: Refusal, m: String| {
            this.stats.refusals += 1;
            Err((r, m))
        };
        if h.version != PROTOCOL {
            return refused(
                self,
                Refusal::Version,
                format!("this server speaks protocol {PROTOCOL}, not {}", h.version),
            );
        }
        if self.draining {
            return refused(
                self,
                Refusal::Draining,
                "the server is draining: connect again through the provider".into(),
            );
        }
        if h.authority != Authority::Validate {
            return refused(
                self,
                Refusal::Version,
                "this server checks movement (validate); it does not simulate it".into(),
            );
        }
        if c.world.as_ref().is_some_and(|w| *w != h.world) {
            return refused(
                self,
                Refusal::Unauthorized,
                format!("the token is not good for world `{}`", h.world),
            );
        }
        if !self.config.serves(&h.world) {
            return refused(
                self,
                Refusal::NotFound,
                format!("this server serves no world `{}`", h.world),
            );
        }
        if usize::from(h.fields_len) > MAX_FIELDS {
            return refused(
                self,
                Refusal::Invalid,
                format!("at most {MAX_FIELDS} bytes of the game's fields"),
            );
        }
        if let Some((_, w)) = self.worlds.get(&h.world)
            && (w.shape != h.shape || w.fields_len != h.fields_len)
        {
            return refused(
                self,
                Refusal::Shape,
                "the game's avatar shape differs from the world's".into(),
            );
        }
        // A lost client coming back takes its avatar again.
        if let Some(sid) = h.resume
            && self.avatars.get(&sid).is_some_and(|a| {
                a.player == c.player
                    && self
                        .worlds
                        .get(&h.world)
                        .is_some_and(|(i, _)| *i == a.world)
            })
        {
            let v = self
                .viewers
                .get_mut(&sid)
                .expect("every avatar has a viewer");
            if v.connected {
                self.closed.push(sid);
            }
            *v = Viewer::new();
            self.stats.joins += 1;
            let a = &self.avatars[&sid];
            let welcome = self.welcome(sid, a, true);
            self.viewers.get_mut(&sid).unwrap().out.push(welcome);
            return Ok(sid);
        }
        // A player joins once: a second join replaces the first.
        if let Some(old) = self
            .avatars
            .iter()
            .find(|(_, a)| a.player == c.player)
            .map(|(s, _)| *s)
        {
            self.remove(old);
            self.closed.push(old);
        }
        if self.avatars.len() >= self.config.max_players {
            return refused(
                self,
                Refusal::Full,
                format!("the server holds its {} players", self.config.max_players),
            );
        }
        let world = match self.worlds.get_mut(&h.world) {
            Some((i, w)) => {
                w.members += 1;
                *i
            }
            None => {
                let i = self.next_world;
                self.next_world += 1;
                self.worlds.insert(
                    h.world.clone(),
                    (
                        i,
                        WorldShape {
                            shape: h.shape,
                            fields_len: h.fields_len,
                            members: 1,
                        },
                    ),
                );
                i
            }
        };
        let sid = loop {
            self.session_count += 1;
            let s = splitmix(self.session_seed ^ self.session_count) | 1;
            if !self.avatars.contains_key(&s) {
                break s;
            }
        };
        let name = if c.name.is_empty() {
            c.player.clone()
        } else {
            c.name
        };
        let kind = c.kind.unwrap_or(h.kind);
        let cap = self.credit_cap(kind);
        let slot = self.free_slots.pop().unwrap_or_else(|| {
            self.slot_gens.push(0);
            (self.slot_gens.len() - 1) as u32
        });
        self.slot_gens[slot as usize] += 1;
        let a = Avatar {
            slot,
            world,
            world_name: h.world.clone(),
            player: c.player,
            name,
            kind,
            fields_len: h.fields_len,
            state: None,
            jumps: 0,
            fields_at: 0,
            credit: cap,
            last_seq: 0,
            received: VecDeque::new(),
            events: self.config.events_per_second as f32,
            last_correct: None,
        };
        let welcome = self.welcome(sid, &a, false);
        let mut v = Viewer::new();
        v.out.push(welcome);
        self.avatars.insert(sid, a);
        self.viewers.insert(sid, v);
        self.stats.joins += 1;
        Ok(sid)
    }

    fn welcome(&self, session: u64, a: &Avatar, resumed: bool) -> Vec<u8> {
        ServerMsg::Welcome(Welcome {
            session,
            player: a.player.clone(),
            name: a.name.clone(),
            tick: self.tick,
            tick_hz: self.config.tick_hz as u8,
            cell_size: self.config.cell_size as f32,
            max_visible: self.config.max_visible as u16,
            max_event: MAX_EVENT as u16,
            authority: Authority::Validate,
            resumed,
        })
        .to_bytes()
    }

    fn credit_cap(&self, kind: u32) -> f32 {
        self.config.rules(kind).max_speed * 0.5 + 1.0
    }

    /// The connection of `session` is gone: its avatar lingers for `lingerTicks`.
    pub fn disconnect(&mut self, session: u64) {
        if let Some(v) = self.viewers.get_mut(&session) {
            v.connected = false;
            v.lost_at = self.tick;
            v.out.clear();
            v.set.clear();
            v.objects.clear();
        }
    }

    fn remove(&mut self, session: u64) {
        if let Some(a) = self.avatars.remove(&session) {
            self.free_slots.push(a.slot);
            self.viewers.remove(&session);
            let name = self
                .worlds
                .iter()
                .find(|(_, (i, _))| *i == a.world)
                .map(|(n, _)| n.clone());
            if let Some(n) = name {
                let w = &mut self.worlds.get_mut(&n).unwrap().1;
                w.members -= 1;
                if w.members == 0 {
                    self.worlds.remove(&n);
                }
            }
        }
    }

    /// Sessions to close now; their connections were replaced.
    pub fn take_closed(&mut self) -> Vec<u64> {
        std::mem::take(&mut self.closed)
    }

    /// What to send `session` now.
    pub fn take_out(&mut self, session: u64) -> Vec<Vec<u8>> {
        self.viewers
            .get_mut(&session)
            .map(|v| std::mem::take(&mut v.out))
            .unwrap_or_default()
    }

    /// The sessions connected now.
    /// Hold each snapshot to `bytes` (a datagram's size): what does not fit
    /// waits, highest priority first, as the byte budget makes it wait.
    pub fn cap_snapshots(&mut self, session: u64, bytes: usize) {
        if let Some(v) = self.viewers.get_mut(&session) {
            v.cap = bytes;
        }
    }

    pub fn sessions(&self) -> impl Iterator<Item = u64> + '_ {
        self.viewers
            .iter()
            .filter(|(_, v)| v.connected)
            .map(|(s, _)| *s)
    }

    /// The avatars `session` holds now (its set's size) and those in its area not in it.
    pub fn in_sight(&self, session: u64) -> (usize, u32) {
        self.viewers
            .get(&session)
            .map_or((0, 0), |v| (v.set.len(), v.others))
    }

    /// A message from `session`'s connection. An error is a malformed message:
    /// the server closes the connection.
    pub fn receive(&mut self, session: u64, bytes: &[u8]) -> Result<(), String> {
        let Some(a) = self.avatars.get(&session) else {
            return Err("no such session".into());
        };
        let msg = ClientMsg::decode(bytes, |s| {
            a.received.iter().find(|(n, _)| *n == s).map(|(_, st)| *st)
        })
        .map_err(|e| e.to_string())?;
        match msg {
            ClientMsg::Hello(_) => return Err("a second hello".into()),
            ClientMsg::Propose(p) => {
                self.stats.proposals += 1;
                let v = self.viewers.get_mut(&session).unwrap();
                if p.ack_tick > 0
                    && v.ack.is_none_or(|(n, _)| p.ack_tick > n)
                    && p.ack_tick <= self.tick
                {
                    v.ack = Some((p.ack_tick, p.ack_bits));
                }
                let a = self.avatars.get_mut(&session).unwrap();
                if p.seq <= a.last_seq {
                    return Ok(());
                }
                a.last_seq = p.seq;
                a.received.push_back((p.seq, p.state));
                while a.received.len() > ACK_WINDOW as usize {
                    a.received.pop_front();
                }
                self.check(session, p.seq, p.state, p.teleport);
            }
            ClientMsg::Event { channel, data } => {
                let a = self.avatars.get_mut(&session).unwrap();
                if a.events < 1.0 || a.state.is_none() {
                    return Ok(());
                }
                a.events -= 1.0;
                self.stats.events += 1;
                self.deliver(session, channel, &data);
            }
            ClientMsg::Ping { time } => {
                let tick = self.tick;
                self.viewers
                    .get_mut(&session)
                    .unwrap()
                    .out
                    .push(ServerMsg::Pong { time, tick }.to_bytes());
            }
            ClientMsg::Leave => self.remove(session),
            ClientMsg::Object(op) => self.object_op(session, op),
        }
        Ok(())
    }

    fn check(&mut self, session: u64, seq: u32, s: AvatarState, teleport: bool) {
        let cs = self.config.cell_size;
        let tick = self.tick;
        let cap_rule = self.config.rules(self.avatars[&session].kind);
        let cap = self.credit_cap(self.avatars[&session].kind);
        let (bmin, bmax, policy) = (
            self.config.bounds_min,
            self.config.bounds_max,
            self.config.teleport,
        );
        let correct_every = (self.config.tick_hz / 4).max(1);
        let a = self.avatars.get_mut(&session).unwrap();
        let pos = s.position(cs);
        let speed = s.velocity().iter().map(|v| v * v).sum::<f32>().sqrt();
        let fits = (0..3).all(|i| (bmin[i]..=bmax[i]).contains(&pos[i]))
            && speed <= cap_rule.max_speed * 1.25 + 1.0
            && s.fields_len == a.fields_len;
        let take = fits
            && match a.state {
                None => true,
                Some(_) if teleport => policy == TeleportPolicy::Any,
                Some(prev) => {
                    let d = dist2(pos, prev.position(cs)).sqrt() as f32;
                    if d <= a.credit + 0.01 {
                        a.credit -= d;
                        true
                    } else {
                        false
                    }
                }
            };
        if take {
            if a.state.is_none() || teleport {
                a.jumps += 1;
                a.credit = cap;
            }
            if a.state.is_none_or(|p| p.fields() != s.fields()) {
                a.fields_at = tick + 1;
            }
            a.state = Some(s);
            return;
        }
        self.stats.rejected += 1;
        if let Some(prev) = a.state
            && a.last_correct.is_none_or(|t| tick - t >= correct_every)
        {
            a.last_correct = Some(tick);
            self.stats.corrections += 1;
            let m = ServerMsg::Correct { seq, state: prev }.to_bytes();
            self.viewers.get_mut(&session).unwrap().out.push(m);
        }
    }

    /// An event from `from` to every connected client whose set holds it.
    fn deliver(&mut self, from: u64, channel: u8, data: &[u8]) {
        let a = &self.avatars[&from];
        let Some(s) = a.state else { return };
        let (w, r) = (a.world, self.config.radius + 1);
        let (slot, generation) = (a.slot, self.slot_gens[a.slot as usize]);
        for dx in -r..=r {
            for dz in -r..=r {
                for (sid, _, _) in self
                    .grid
                    .get(&(w, s.cell[0] + dx, s.cell[2] + dz))
                    .into_iter()
                    .flatten()
                {
                    if let Some(v) = self.viewers.get_mut(sid)
                        && v.connected
                        && let Some(seen) = v
                            .set
                            .iter()
                            .find(|x| x.slot == slot && x.generation == generation)
                    {
                        v.out.push(
                            ServerMsg::Event {
                                from: seen.id,
                                channel,
                                data: data.to_vec(),
                            }
                            .to_bytes(),
                        );
                    }
                }
            }
        }
    }

    /// One tick: lingering avatars go, and every connected client gets its
    /// leaves, enters and snapshot. Clients are independent, so they are
    /// split across the machine's cores.
    pub fn step(&mut self) {
        self.tick += 1;
        let tick = self.tick;
        let linger = self.config.linger_ticks;
        let gone: Vec<u64> = self
            .viewers
            .iter()
            .filter(|(_, v)| !v.connected && tick - v.lost_at > linger)
            .map(|(s, _)| *s)
            .collect();
        for s in gone {
            self.remove(s);
        }
        let hz = self.config.tick_hz as f32;
        for a in self.avatars.values_mut() {
            let rules = self.config.rules(a.kind);
            a.credit = (a.credit + rules.max_speed / hz).min(rules.max_speed * 0.5 + 1.0);
            a.events = (a.events + self.config.events_per_second as f32 / hz)
                .min(self.config.events_per_second as f32);
        }
        for cell in self.grid.values_mut() {
            cell.clear();
        }
        let cs = self.config.cell_size;
        for (sid, a) in &self.avatars {
            if let Some(s) = a.state {
                self.grid
                    .entry((a.world, s.cell[0], s.cell[2]))
                    .or_default()
                    .push((*sid, a.slot, s.position(cs)));
            }
        }
        self.grid.retain(|_, c| !c.is_empty());
        self.cur.clear();
        self.cur.resize(self.slot_gens.len(), Cur::default());
        self.alive.clear();
        self.alive.resize(self.slot_gens.len(), (0, 0));
        for a in self.avatars.values() {
            self.cur[a.slot as usize] = Cur {
                jumps: a.jumps,
                fields_at: a.fields_at,
                state: a.state,
            };
            if a.state.is_some() {
                self.alive[a.slot as usize] = (self.slot_gens[a.slot as usize], a.world);
            }
        }
        let row = &mut self.ring[tick as usize % RING];
        row.resize(self.slot_gens.len(), Motion::default());
        for a in self.avatars.values() {
            if let Some(st) = a.state {
                row[a.slot as usize] = Motion::of(&st);
            }
        }
        self.objects_tick();
        let ctx = TickCtx {
            ring: &self.ring,
            cur: &self.cur,
            alive: &self.alive,
            slot_gens: &self.slot_gens,
            avatars: &self.avatars,
            grid: &self.grid,
            config: &self.config,
            tick,
        };
        let mut viewers: Vec<(u64, &mut Viewer)> = self
            .viewers
            .iter_mut()
            .filter(|(_, v)| v.connected)
            .map(|(s, v)| (*s, v))
            .collect();
        // A persistent work-stealing pool: no thread is made per tick, and a
        // thread the OS preempts (socket work shares these cores) holds up
        // only the clients it has taken.
        let counts = if viewers.len() >= 128 {
            use rayon::prelude::*;
            viewers
                .par_iter_mut()
                .with_min_len(16)
                .map(|(sid, v)| {
                    SCRATCH.with_borrow_mut(|scratch| ctx.viewer_tick(*sid, v, scratch))
                })
                .reduce(|| [0u64; 3], |a, c| [a[0] + c[0], a[1] + c[1], a[2] + c[2]])
        } else {
            SCRATCH.with_borrow_mut(|scratch| {
                viewers.iter_mut().fold([0u64; 3], |acc, (sid, v)| {
                    let c = ctx.viewer_tick(*sid, v, scratch);
                    [acc[0] + c[0], acc[1] + c[1], acc[2] + c[2]]
                })
            })
        };
        self.stats.enters += counts[0];
        self.stats.leaves += counts[1];
        self.stats.updates += counts[2];
    }
}

/// Shared objects (docs/design.md §2.12): server-owned, changed by clients'
/// ops in the order they arrive, and sent through the same interest area as
/// avatars. A client may change an object only within its radius (where it
/// was and where it goes), at most `eventsPerSecond` ops a second with its
/// events, and only while the object has the version it expects, if it
/// names one. Every client that holds the object, or now has it within its
/// radius, hears the new state at once; one whose sight it left hears it
/// gone. Each client's set of objects is chosen again on its refresh ticks:
/// those within `radius` cells come in, those past `radius + 1` go.
impl Shard {
    /// The objects a store kept, before the first join.
    pub fn load_objects(&mut self, objects: impl IntoIterator<Item = (String, SharedObject)>) {
        for (world, o) in objects {
            self.objects.entry(world).or_default().put(o);
        }
    }

    /// The changes since the last call, in order, for a store.
    pub fn take_changes(&mut self) -> Vec<Change> {
        std::mem::take(&mut self.changes)
    }

    /// The object `key` of `world`, as the shard holds it.
    pub fn object(&self, world: &str, key: &str) -> Option<&SharedObject> {
        self.objects.get(world)?.by_key.get(key)
    }

    fn object_op(&mut self, session: u64, op: ObjectOp) {
        let r = self.config.radius;
        let (cs, bmin, bmax) = (
            self.config.cell_size,
            self.config.bounds_min,
            self.config.bounds_max,
        );
        let a = self.avatars.get_mut(&session).expect("checked by receive");
        let world = a.world_name.clone();
        let current = self
            .objects
            .get(&world)
            .and_then(|w| w.by_key.get(&op.key))
            .cloned();
        let have = current.as_ref().map_or(0, |o| o.version);
        let me = a.state.map(|s| s.cell);
        let fits = crate::protocol::object_key(&op.key)
            && op.expect.is_none_or(|v| v == have)
            && me.is_some_and(|me| {
                current.as_ref().is_none_or(|o| near(me, o.place.cell, r))
                    && op.set.as_ref().is_none_or(|(p, _)| {
                        let at = p.position(cs);
                        near(me, p.cell, r) && (0..3).all(|i| (bmin[i]..=bmax[i]).contains(&at[i]))
                    })
            })
            && (op.set.is_some() || current.is_some())
            && a.events >= 1.0;
        let answer = |taken, version| {
            ServerMsg::Op {
                seq: op.seq,
                taken,
                version,
            }
            .to_bytes()
        };
        if !fits {
            self.stats.ops_refused += 1;
            let m = answer(false, have);
            self.viewers.get_mut(&session).unwrap().out.push(m);
            return;
        }
        a.events -= 1.0;
        self.stats.ops += 1;
        let objects = self.objects.entry(world.clone()).or_default();
        let (version, now) = match op.set {
            Some((place, data)) => {
                let o = SharedObject {
                    key: op.key.clone(),
                    version: have + 1,
                    place,
                    data,
                };
                objects.put(o.clone());
                self.changes.push(Change::Put {
                    world: world.clone(),
                    object: o.clone(),
                });
                (o.version, Some(o))
            }
            None => {
                objects.remove(&op.key);
                if objects.by_key.is_empty() {
                    self.objects.remove(&world);
                }
                self.changes.push(Change::Remove {
                    world: world.clone(),
                    key: op.key.clone(),
                });
                (0, None)
            }
        };
        let m = answer(true, version);
        self.viewers.get_mut(&session).unwrap().out.push(m);
        // Everyone who holds it, or now has it in its radius, hears at once.
        let (whole, gone) = (
            now.as_ref()
                .map(|o| ServerMsg::Object(o.clone()).to_bytes()),
            ServerMsg::ObjectGone {
                key: op.key.clone(),
            }
            .to_bytes(),
        );
        for (sid, v) in self.viewers.iter_mut().filter(|(_, v)| v.connected) {
            let a = &self.avatars[sid];
            let Some(cell) = a.state.map(|s| s.cell) else {
                continue;
            };
            if a.world_name != world {
                continue;
            }
            match (&now, &whole) {
                (Some(o), Some(m)) if near(cell, o.place.cell, r) => {
                    v.objects.insert(op.key.clone(), o.version);
                    v.out.push(m.clone());
                }
                _ => {
                    if v.objects.remove(&op.key).is_some() {
                        v.out.push(gone.clone());
                    }
                }
            }
        }
    }

    /// Each client on its refresh tick: the objects within its radius it
    /// does not hold come in; those it holds past `radius + 1` go.
    fn objects_tick(&mut self) {
        // No objects: none is held either (a removal told its holders).
        if self.objects.is_empty() {
            return;
        }
        let (tick, r) = (self.tick, self.config.radius);
        for (sid, v) in self.viewers.iter_mut().filter(|(_, v)| v.connected) {
            if !(v.objects.is_empty() || sid.wrapping_add(u64::from(tick)) % REFRESH_TICKS == 0) {
                continue;
            }
            let a = &self.avatars[sid];
            let Some(cell) = a.state.map(|s| s.cell) else {
                continue;
            };
            let objects = self.objects.get(&a.world_name);
            v.objects.retain(|key, _| {
                let keep = objects
                    .and_then(|w| w.by_key.get(key))
                    .is_some_and(|o| near(cell, o.place.cell, r + 1));
                if !keep {
                    v.out
                        .push(ServerMsg::ObjectGone { key: key.clone() }.to_bytes());
                }
                keep
            });
            let Some(objects) = objects else { continue };
            for dx in -r..=r {
                for dz in -r..=r {
                    let Some(keys) = objects.cells.get(&(cell[0] + dx, cell[2] + dz)) else {
                        continue;
                    };
                    for key in keys {
                        if !v.objects.contains_key(key) {
                            let o = &objects.by_key[key];
                            v.objects.insert(key.clone(), o.version);
                            v.out.push(ServerMsg::Object(o.clone()).to_bytes());
                        }
                    }
                }
            }
        }
    }
}

impl Shard {
    /// Touch what the next step reads, on the pool that runs it: a step
    /// after a tick's idle gap otherwise starts on parked pool threads and
    /// cold caches, and took about three times one run back to back (p99
    /// 20 ms against 3.5 ms at 2000 clients on 4 vCPU). Changes nothing.
    pub fn warm(&self) {
        let touch = |v: &Viewer| {
            let mut n = 0u64;
            for seen in &v.set {
                n = n.wrapping_add(u64::from(
                    self.slot_gens.get(seen.slot as usize).copied().unwrap_or(0),
                ));
                if let Some(at) = seen.sent.ticks(self.tick + 1).last()
                    && let Some(m) = self.ring[at as usize % RING].get(seen.slot as usize)
                {
                    n = n.wrapping_add(u64::from(m.yaw));
                }
            }
            n
        };
        let viewers: Vec<&Viewer> = self.viewers.values().filter(|v| v.connected).collect();
        let n: u64 = if viewers.len() >= 128 {
            use rayon::prelude::*;
            viewers.par_iter().with_min_len(16).map(|v| touch(v)).sum()
        } else {
            viewers.iter().map(|v| touch(v)).sum()
        };
        std::hint::black_box(n);
    }
}

/// A thread's reused buffers for client ticks.
#[derive(Default)]
struct Scratch {
    /// Distance (squared, members' shortened), session and slot.
    candidates: Vec<(f64, u64, u32)>,
    /// Priority bits above `u32::MAX - index in the set`: sorted descending,
    /// the highest priority first and the nearer of equals first.
    due: Vec<u64>,
    body: Vec<u8>,
    /// By slot: the `stamp` of the set choice that marked it a member, and
    /// its index in the set then.
    mark: Vec<(u32, u16)>,
    stamp: u32,
    keep: Vec<bool>,
    spare: Vec<Seen>,
}

thread_local! {
    static SCRATCH: std::cell::RefCell<Scratch> = std::cell::RefCell::default();
}

/// What one tick's clients read: shared, never written while they run.
struct TickCtx<'a> {
    cur: &'a [Cur],
    alive: &'a [(u32, u32)],
    ring: &'a [Vec<Motion>],
    slot_gens: &'a [u32],
    avatars: &'a Fast<u64, Avatar>,
    grid: &'a Grid,
    config: &'a Config,
    tick: u32,
}

impl TickCtx<'_> {
    /// One client's tick: its set, its leaves and enters, its snapshot.
    /// Returns the enters, leaves and updates it sent.
    fn viewer_tick(&self, sid: u64, v: &mut Viewer, scratch: &mut Scratch) -> [u64; 3] {
        let (tick, cs) = (self.tick, self.config.cell_size);
        let (r, max_visible) = (self.config.radius, self.config.max_visible);
        let budget = (self.config.bytes_per_second / self.config.tick_hz as usize).min(v.cap);
        let me = &self.avatars[&sid];
        let (mut enters, mut leaves, mut updates) = (0, 0, 0);
        // Who is in sight: chosen again every REFRESH_TICKS ticks, staggered
        // by session, and every tick while the set is empty. Between, a
        // member keeps its rank, and one whose avatar went (or moved to
        // another world) leaves at once.
        let refresh = v.set.is_empty() || sid.wrapping_add(u64::from(tick)) % REFRESH_TICKS == 0;
        if refresh {
            scratch.stamp = scratch.stamp.wrapping_add(1).max(1);
            if scratch.stamp == 1 {
                scratch.mark.clear();
            }
            let stamp = scratch.stamp;
            let mark = &mut scratch.mark;
            if mark.len() < self.slot_gens.len() {
                mark.resize(self.slot_gens.len(), (0, 0));
            }
            for (i, seen) in v.set.iter().enumerate() {
                if self.slot_gens[seen.slot as usize] == seen.generation {
                    mark[seen.slot as usize] = (stamp, i as u16);
                }
            }
            let candidates = &mut scratch.candidates;
            candidates.clear();
            let mut within = 0u32;
            if let Some(s) = me.state {
                let here = s.position(cs);
                for dx in -(r + 1)..=(r + 1) {
                    for dz in -(r + 1)..=(r + 1) {
                        let ring = dx.abs().max(dz.abs());
                        let Some(cell) = self.grid.get(&(me.world, s.cell[0] + dx, s.cell[2] + dz))
                        else {
                            continue;
                        };
                        for &(other, slot, pos) in cell {
                            if other == sid {
                                continue;
                            }
                            let member = mark[slot as usize].0 == stamp;
                            if ring > r && !member {
                                continue;
                            }
                            if ring <= r {
                                within += 1;
                            }
                            let d = dist2(here, pos);
                            candidates.push((if member { d * 0.8 } else { d }, other, slot));
                        }
                    }
                }
            }
            let by =
                |a: &(f64, u64, u32), b: &(f64, u64, u32)| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1));
            if candidates.len() > max_visible {
                candidates.select_nth_unstable_by(max_visible, by);
                candidates.truncate(max_visible);
            }
            candidates.sort_unstable_by(by);
            // The members that stay, then leaves (in the old order), then
            // the new set in rank order with its enters.
            let old = std::mem::take(&mut v.set);
            let keep = &mut scratch.keep;
            keep.clear();
            keep.resize(old.len(), false);
            for &(_, _, slot) in candidates.iter() {
                let (st, i) = mark[slot as usize];
                if st == stamp {
                    keep[usize::from(i)] = true;
                }
            }
            for (seen, _) in old.iter().zip(keep.iter()).filter(|(_, k)| !**k) {
                v.out.push(ServerMsg::Leave { id: seen.id }.to_bytes());
                leaves += 1;
            }
            let mut set = std::mem::take(&mut scratch.spare);
            set.clear();
            for (rank, &(_, s, slot)) in candidates.iter().enumerate() {
                let (st, i) = mark[slot as usize];
                if st == stamp {
                    let mut seen = old[usize::from(i)];
                    seen.rank = rank as u16;
                    seen.priority += weight(rank);
                    set.push(seen);
                    continue;
                }
                let id = loop {
                    let id = v.next_id;
                    v.next_id = v.next_id.wrapping_add(1);
                    let kept = old.iter().zip(keep.iter()).any(|(x, k)| *k && x.id == id);
                    if !kept && !set.iter().any(|x| x.id == id) {
                        break id;
                    }
                };
                let a = &self.avatars[&s];
                v.out.push(
                    ServerMsg::Enter(Enter {
                        id,
                        tick,
                        player: a.player.clone(),
                        name: a.name.clone(),
                        kind: a.kind,
                        state: a.state.unwrap(),
                    })
                    .to_bytes(),
                );
                set.push(Seen {
                    id,
                    rank: rank as u16,
                    enter_tick: tick,
                    sent: Sent::new(),
                    priority: 0.0,
                    jumps: a.jumps as u16,
                    slot,
                    generation: self.slot_gens[slot as usize],
                });
                enters += 1;
            }
            v.set = set;
            scratch.spare = old;
            v.others = within.saturating_sub(v.set.len() as u32);
        } else {
            let world = me.world;
            v.set.retain(|x| {
                let gone = self.alive[x.slot as usize] != (x.generation, world);
                if gone {
                    v.out.push(ServerMsg::Leave { id: x.id }.to_bytes());
                    leaves += 1;
                }
                !gone
            });
            for seen in v.set.iter_mut() {
                seen.priority += weight(usize::from(seen.rank));
            }
        }
        // The snapshot.
        let base = v.ack.filter(|(n, _)| tick - n <= ACK_WINDOW);
        let due = &mut scratch.due;
        due.clear();
        due.extend(
            v.set
                .iter()
                .enumerate()
                .filter(|(_, x)| x.priority >= 1.0)
                .map(|(i, x)| {
                    u64::from(x.priority.to_bits()) << 32 | u64::from(u32::MAX - i as u32)
                }),
        );
        due.sort_unstable_by(|a, b| b.cmp(a));
        let mut body = Writer {
            buf: std::mem::take(&mut scratch.body),
        };
        body.buf.clear();
        let mut count = 0;
        for &k in due.iter() {
            let seen = &mut v.set[(u32::MAX - k as u32) as usize];
            let a = &self.cur[seen.slot as usize];
            let state = a.state.unwrap();
            let b = base.and_then(|(n, bits)| {
                let ticks = seen.sent.ticks(tick);
                let i = baseline_index(ticks, seen.enter_tick, n, |t| acked(n, bits, t))?;
                seen.sent.drop_front(i);
                // The row of a tick it was sent at holds its slot.
                let at = seen.sent.ticks(tick).next()?;
                Some((self.ring[at as usize % RING][seen.slot as usize], at))
            });
            let fields_moved = b.is_some_and(|(_, at)| a.fields_at > at);
            let b = b.map(|(motion, _)| motion.with_fields_of(&state));
            let jump = a.jumps as u16 != seen.jumps;
            let mut m = state.diff(b.as_ref());
            if fields_moved {
                m |= mask::FIELDS;
            }
            if m == 0 && !jump {
                seen.priority = 0.0;
                continue;
            }
            if jump {
                m |= mask::JUMP;
            }
            if b.is_none() {
                m |= mask::WHOLE;
            }
            let before = body.len();
            body.varint(u64::from(seen.id));
            body.u8(m);
            state.write_parts(m & mask::STATE, b.as_ref(), &mut body);
            if count > 0 && body.len() + 16 > budget {
                body.buf.truncate(before);
                break;
            }
            seen.jumps = a.jumps as u16;
            seen.priority = 0.0;
            seen.sent.push(tick);
            count += 1;
        }
        // What a send no longer needs is dropped as baselines are found; this
        // drops what is past the window, on the ticks the set is chosen.
        let floor = tick.saturating_sub(2 * ACK_WINDOW);
        for seen in v.set.iter_mut().filter(|_| refresh) {
            let old = seen.sent.ticks(tick).take_while(|t| *t < floor).count();
            seen.sent.drop_front(old);
        }
        updates += count as u64;
        let mut w = Writer {
            buf: Vec::with_capacity(16 + body.len()),
        };
        SnapshotHead {
            tick,
            base: base.map(|(n, _)| n),
            ack_seq: me.last_seq,
            others: v.others,
        }
        .write(count, &mut w);
        w.buf.extend_from_slice(&body.buf);
        scratch.body = body.buf;
        v.out.push(w.buf);
        [enters, leaves, updates]
    }
}

impl Viewer {
    fn new() -> Viewer {
        Viewer {
            connected: true,
            lost_at: 0,
            ack: None,
            set: Vec::new(),
            next_id: 0,
            out: Vec::new(),
            others: 0,
            cap: usize::MAX,
            objects: HashMap::new(),
        }
    }
}
