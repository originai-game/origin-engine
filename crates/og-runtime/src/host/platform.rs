//! The IDL's `platform` module (docs/design.md §2.8): calls game code makes
//! by method name, answered by the host. A call is queued with a request
//! number; the host takes the queue (`og_platform_take`), answers each call
//! (`og_platform_answer`: the web shell through the Player, the native host
//! from its local JSON store), and the answers are applied by the
//! `og.platform` system at the start of the next tick, which sends
//! `og.PlatformAnswered`. Every call and answer stays in the engine's log
//! for `og.platform.result` and the probe's `dump_platform`.
//!
//! Time: the host tells the engine its wall clock (UTC ms) when it attaches
//! and with every answer (`time`); `og.platform.now` is the last of those,
//! moved on by the ticks since, so game code reads a wall clock only as
//! host input, which a replay with the same answers repeats.

use crate::engine::{engine, running};
use crate::host::wire::{PlatformError, PlatformInfo, PlatformNoticeData, PlatformResult, ToJson};
use crate::parse;
use og_core::IdMap;
use og_core::Reflect;
use og_ecs::{SystemDesc, World};
use serde_json::{Value, json};
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};

mod services;
pub(crate) use services::{GRANTS, enable_grants};
pub use services::{
    og_platform_achievement_progress, og_platform_achievement_unlock, og_platform_achievements,
    og_platform_ad_ready, og_platform_consume, og_platform_entitlements, og_platform_friends,
    og_platform_grant, og_platform_granted, og_platform_invite_send, og_platform_invites,
    og_platform_player, og_platform_products, og_platform_purchase, og_platform_share,
    og_platform_show_ad, og_platform_supports,
};

mod calls;
mod host_side;
mod results;
mod slots;
mod system;
mod time;

pub use calls::*;
pub use host_side::*;
pub use results::*;
pub use slots::*;
pub(crate) use system::*;
pub use time::*;

/// The system's name in the schedule (before the game's systems).
pub const PLATFORM_SYSTEM: &str = "og.platform";

/// `og.PlatformAnswered`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Reflect)]
#[reflect(name = "og.PlatformAnswered")]
pub struct PlatformAnswered {
    pub request: u32,
    pub ok: bool,
}

/// The most calls kept in the log (the oldest answered ones go first).
const MAX_LOG: usize = 512;
/// The most calls waiting for an answer: a call past it is refused, so
/// unanswered calls (which the log cannot drop) stay under [`MAX_LOG`].
const MAX_PENDING: usize = 256;
/// The largest stored value, encoded.
const MAX_VALUE: usize = 64 * 1024;
/// The largest answer a host may give, encoded: a stored value or a
/// leaderboard page with room to spare. A longer one answers its call
/// with a `too_large` error.
const MAX_ANSWER: usize = 4 * MAX_VALUE;

const RETRYABLE: &[&str] = &[
    "offline",
    "timeout",
    "unavailable",
    "rate_limited",
    "no_fill",
];

/// `og.PlatformNotice`'s name: a lazy record (crate::engine::lazy), registered when
/// a game first names it.
pub const NOTICE: &str = "og.PlatformNotice";
/// og.PlatformAnswered's name.
const ANSWERED: &str = "og.PlatformAnswered";
/// The notices kept for og.platform.notice.
const KEEP_NOTICES: usize = 256;
/// What a notice may be about.
const NOTICE_KINDS: &[&str] = &["purchase", "entitlements", "invite", "host"];

/// Whether `event` is og.platform's own: an answer or a notice, which a
/// system that writes shared state in a lockstep session never hears
/// ([`withheld`]).
pub(crate) fn is_platform_event(event: &str) -> bool {
    event == ANSWERED || event == NOTICE
}

/// One notice the host sent (og_platform_notify).
struct Notice {
    kind: String,
    data: Value,
    time: f64,
    /// The tick it was applied at.
    tick: u64,
}

struct Record {
    method: String,
    args: Value,
    tick: u64,
    /// `{ ok, data }` or `{ ok: false, error }`, and the tick it was applied at.
    answer: Option<(Value, u64)>,
    /// Fire and forget (telemetry): never answered.
    oneway: bool,
    /// A slot call (saveSlot, loadSlot): answered by the engine when its
    /// store calls are, never sent to the host.
    internal: bool,
}

/// A slot call in progress: the store calls it made and what they answered.
struct SlotCall {
    slot: String,
    stage: Stage,
}

enum Stage {
    /// Both: the manifest's store.get is in flight.
    Manifest(SlotOp),
    /// saveSlot: the parts' store.set calls left, then the manifest to write.
    Parts {
        left: BTreeSet<u32>,
        manifest: Value,
        revision: u64,
    },
    /// saveSlot: the manifest's store.set.
    Write { parts: usize, bytes: usize },
    /// loadSlot: the parts' store.get calls, in order, and their texts.
    Read {
        parts: Vec<u32>,
        got: IdMap<u32, String>,
        manifest: Value,
        revision: u64,
    },
}

enum SlotOp {
    Save { text: String, meta: Value },
    Load { meta_only: bool },
}

#[derive(Default)]
pub(crate) struct Platform {
    next: u32,
    outbox: Vec<u32>,
    inbox: Vec<(u32, Value)>,
    log: IdMap<u32, Record>,
    host: Option<String>,
    capabilities: Vec<String>,
    /// PlatformInfo.clock: whose time `now` reads.
    clock: Option<String>,
    /// The host's last time (UTC ms) and the tick it held at: `now` moves
    /// it on by the ticks since. Set when the host attaches and by every
    /// answer that carries a time, at the tick's start the answer is applied.
    base: Option<(f64, u64)>,
    /// Slot calls in progress by request, and the store calls each made.
    slots: IdMap<u32, SlotCall>,
    sub: IdMap<u32, u32>,
    /// Notices sent since the last tick, applied at the next tick's start.
    notices_in: Vec<Value>,
    /// Applied notices by number (the last [`KEEP_NOTICES`]).
    notices: BTreeMap<u32, Notice>,
    next_notice: u32,
}

/// og.platform on one engine: its calls, answers and notices.
pub(crate) type ModuleState = RefCell<Platform>;

fn with<T>(engine: u32, f: impl FnOnce(&mut Platform) -> T) -> T {
    match crate::engine::engine(engine) {
        Ok(eng) => f(&mut eng.modules.platform.borrow_mut()),
        // An engine that is gone, or not made yet, has none: a fresh one.
        Err(_) => f(&mut Platform::default()),
    }
}

/// The tick a call is made at: the running system's, or the world's.
fn tick_now(world: u32) -> Result<u64, String> {
    if let Some(f) = running(world) {
        // SAFETY: the frame is published while its system runs.
        return Ok(unsafe { f.ctx() }.time().tick);
    }
    Ok(engine(world)?.world()?.time().tick)
}

fn call(world: u32, method: &str, args: Value, oneway: bool) -> Result<u32, String> {
    let tick = tick_now(world)?;
    with(world, |p| issue(p, method, args, tick, oneway, false))
}

/// Logs a call and queues it for the host (an internal one is the
/// engine's to answer and is not sent).
fn issue(
    p: &mut Platform,
    method: &str,
    args: Value,
    tick: u64,
    oneway: bool,
    internal: bool,
) -> Result<u32, String> {
    if !oneway {
        let waiting = p
            .log
            .values()
            .filter(|r| r.answer.is_none() && !r.oneway)
            .count();
        if waiting >= MAX_PENDING {
            return Err(format!(
                "{method}: {waiting} platform calls are waiting for the host's answers; at most {MAX_PENDING} wait at once"
            ));
        }
    }
    p.next = p.next.wrapping_add(1).max(1);
    let id = p.next;
    p.log.insert(
        id,
        Record {
            method: method.to_string(),
            args,
            tick,
            answer: None,
            oneway,
            internal,
        },
    );
    if !internal {
        p.outbox.push(id);
    }
    if p.log.len() > MAX_LOG {
        let old: Vec<u32> = p
            .log
            .iter()
            .filter(|(_, r)| r.answer.is_some() || r.oneway)
            .map(|(k, _)| *k)
            .take(p.log.len() - MAX_LOG)
            .collect();
        for k in old {
            p.log.remove(&k);
        }
        // A one-way call dropped before the host took it is not sent.
        let log = &p.log;
        p.outbox.retain(|id| log.contains_key(id));
    }
    Ok(id)
}

fn request(world: u32, method: &str, args: Value) -> Result<u32, String> {
    call(world, method, args, false)
}

// ── the system and the probe ─────────────────────────────────────────────

#[cfg(test)]
mod tests;
