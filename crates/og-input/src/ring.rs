//! The shell's input ring: the SharedArrayBuffer the main thread writes page
//! input into and the engine Worker drains once per frame (design §2.2;
//! written by `packages/engine/src/web/input.ts`, whose layout this module
//! decodes and must match).
//!
//! The buffer is `(2 + 256 × 16) × 4` = 16,392 bytes of little-endian `i32`
//! words:
//!
//! | words | what |
//! |---|---|
//! | 0 | write index: events ever written (monotonic; the writer's `Atomics.store` publishes a slot) |
//! | 1 | read index: events ever drained (monotonic; the reader stores it back) |
//! | 2 + 16·s … | slot `s = index % 256` |
//!
//! One slot is 16 words: `kind` (0 key, 1 pointer, 2 wheel), `action` (0
//! press, 1 down, 2 up, 3 click, 4 move), `button` (0 none, 1 left, 2
//! middle, 3 right), the key's UTF-8 length (≤ 32), then `x`, `y`,
//! `deltaX`, `deltaY` as f32 bits, then 32 bytes of the key code (words
//! 8–15). Unused words are 0. The writer refuses an event when
//! `write - read ≥ 256` (the ring is full; the event is dropped), so a
//! reader never sees an overwritten slot. Modifiers do not cross the ring.
//!
//! Indices are compared with wrapping `u32` arithmetic here; the TypeScript
//! writer indexes slots with `%` on an `i32`, which stays correct for the
//! first 2^31 events of a page.

use crate::event::{InjectedInput, KeyAction, Modifiers, PointerAction, PointerButton};

/// Header words (write index, read index).
pub const HEADER_WORDS: usize = 2;
/// Words per slot.
pub const SLOT_WORDS: usize = 16;
/// Slots in the ring.
pub const SLOTS: usize = 256;
/// Bytes of key code a slot holds.
pub const KEY_BYTES: usize = 32;
/// The ring's size in bytes.
pub const BYTE_LEN: usize = (HEADER_WORDS + SLOTS * SLOT_WORDS) * 4;
/// `kind` values, by index.
pub const KINDS: [&str; 3] = ["key", "pointer", "wheel"];
/// `action` values, by index.
pub const ACTIONS: [&str; 5] = ["press", "down", "up", "click", "move"];
/// `button` values, by index.
pub const BUTTONS: [&str; 4] = ["none", "left", "middle", "right"];

fn word(ring: &[u8], i: usize) -> u32 {
    u32::from_le_bytes(ring[i * 4..i * 4 + 4].try_into().expect("4 bytes"))
}

fn set_word(ring: &mut [u8], i: usize, v: u32) {
    ring[i * 4..i * 4 + 4].copy_from_slice(&v.to_le_bytes());
}

/// What [`drain`] took from a ring.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Drained {
    /// The decoded events, oldest first.
    pub events: Vec<InjectedInput>,
    /// Slots that did not decode, each with its index and the reason.
    pub skipped: Vec<String>,
    /// The new read index (the write index read at the start).
    pub read: u32,
}

/// One slot (64 bytes) as an event; the reason when its words name no event.
pub fn decode_slot(slot: &[u8]) -> Result<InjectedInput, String> {
    if slot.len() != SLOT_WORDS * 4 {
        return Err(format!(
            "a slot is {} bytes, got {}",
            SLOT_WORDS * 4,
            slot.len()
        ));
    }
    let w = |i: usize| word(slot, i);
    let f = |i: usize| f32::from_bits(w(i));
    let action = ACTIONS
        .get(w(1) as usize)
        .copied()
        .ok_or_else(|| format!("action {}", w(1) as i32))?;
    match w(0) {
        0 => {
            let len = w(3) as usize;
            if len == 0 || len > KEY_BYTES {
                return Err(format!("key length {}", w(3) as i32));
            }
            let key = std::str::from_utf8(&slot[32..32 + len])
                .map_err(|_| "a key code that is not UTF-8".to_owned())?;
            let action = match action {
                "press" => KeyAction::Press,
                "down" => KeyAction::Down,
                "up" => KeyAction::Up,
                other => return Err(format!("key action '{other}'")),
            };
            Ok(InjectedInput::Key {
                key: key.to_owned(),
                action,
                modifiers: Modifiers::default(),
            })
        }
        1 => {
            let action = match action {
                "click" => PointerAction::Click,
                "move" => PointerAction::Move,
                "down" => PointerAction::Down,
                "up" => PointerAction::Up,
                other => return Err(format!("pointer action '{other}'")),
            };
            let button =
                PointerButton::from_index(w(2)).ok_or_else(|| format!("button {}", w(2) as i32))?;
            let (x, y) = (f(4), f(5));
            if !x.is_finite() || !y.is_finite() {
                return Err("a non-finite pointer position".into());
            }
            Ok(InjectedInput::Pointer {
                action,
                x,
                y,
                button,
            })
        }
        2 => {
            let (x, y, delta_x, delta_y) = (f(4), f(5), f(6), f(7));
            if ![x, y, delta_x, delta_y].iter().all(|v| v.is_finite()) {
                return Err("a non-finite wheel value".into());
            }
            Ok(InjectedInput::Wheel {
                x,
                y,
                delta_x,
                delta_y,
            })
        }
        k => Err(format!("kind {}", k as i32)),
    }
}

/// One event as a slot's 64 bytes, as `writeRing` writes it. The key code
/// is truncated to [`KEY_BYTES`] as the writer does.
pub fn encode_slot(event: &InjectedInput) -> [u8; SLOT_WORDS * 4] {
    let mut slot = [0u8; SLOT_WORDS * 4];
    let index =
        |table: &[&str], name: &str| table.iter().position(|n| *n == name).unwrap_or(0) as u32;
    let (kind, action, button, xy, delta) = match event {
        InjectedInput::Key { key, action, .. } => {
            let bytes = &key.as_bytes()[..key.len().min(KEY_BYTES)];
            set_word(&mut slot, 3, bytes.len() as u32);
            slot[32..32 + bytes.len()].copy_from_slice(bytes);
            let a = match action {
                KeyAction::Press => "press",
                KeyAction::Down => "down",
                KeyAction::Up => "up",
            };
            (0, index(&ACTIONS, a), 0, (0.0, 0.0), (0.0, 0.0))
        }
        InjectedInput::Pointer {
            action,
            x,
            y,
            button,
        } => {
            let a = match action {
                PointerAction::Click => "click",
                PointerAction::Move => "move",
                PointerAction::Down => "down",
                PointerAction::Up => "up",
            };
            (1, index(&ACTIONS, a), button.index(), (*x, *y), (0.0, 0.0))
        }
        InjectedInput::Wheel {
            x,
            y,
            delta_x,
            delta_y,
        } => (2, 0, 0, (*x, *y), (*delta_x, *delta_y)),
    };
    set_word(&mut slot, 0, kind);
    set_word(&mut slot, 1, action);
    set_word(&mut slot, 2, button);
    set_word(&mut slot, 4, xy.0.to_bits());
    set_word(&mut slot, 5, xy.1.to_bits());
    set_word(&mut slot, 6, delta.0.to_bits());
    set_word(&mut slot, 7, delta.1.to_bits());
    slot
}

/// A new, empty ring (all zero).
pub fn new_ring() -> Vec<u8> {
    vec![0; BYTE_LEN]
}

/// Appends an event as the main thread's `writeRing` does; `false` when the
/// ring is full (the event is dropped) or `ring` is not [`BYTE_LEN`] long.
/// For native hosts and tests; the web shell writes from TypeScript.
pub fn write(ring: &mut [u8], event: &InjectedInput) -> bool {
    if ring.len() != BYTE_LEN {
        return false;
    }
    let (w, r) = (word(ring, 0), word(ring, 1));
    if w.wrapping_sub(r) as usize >= SLOTS {
        return false;
    }
    let at = (HEADER_WORDS + (w as usize % SLOTS) * SLOT_WORDS) * 4;
    ring[at..at + SLOT_WORDS * 4].copy_from_slice(&encode_slot(event));
    set_word(ring, 0, w.wrapping_add(1));
    true
}

/// Every event written since the last drain, oldest first, from a copy of
/// the ring's bytes (or the live buffer). Slots that do not decode are
/// skipped with a reason. A header claiming more than [`SLOTS`] unread
/// events (a corrupt or foreign buffer) drains only the newest [`SLOTS`]
/// and says so. The caller stores [`Drained::read`] into word 1.
pub fn drain(ring: &[u8]) -> Drained {
    let mut out = Drained::default();
    if ring.len() != BYTE_LEN {
        out.skipped
            .push(format!("the ring is {} bytes, not {BYTE_LEN}", ring.len()));
        return out;
    }
    let (w, mut r) = (word(ring, 0), word(ring, 1));
    let unread = w.wrapping_sub(r) as usize;
    if unread > SLOTS {
        out.skipped.push(format!(
            "the header claims {unread} unread events; the ring holds {SLOTS}, so the oldest were skipped"
        ));
        r = w.wrapping_sub(SLOTS as u32);
    }
    while r != w {
        let at = (HEADER_WORDS + (r as usize % SLOTS) * SLOT_WORDS) * 4;
        match decode_slot(&ring[at..at + SLOT_WORDS * 4]) {
            Ok(event) => out.events.push(event),
            Err(reason) => out.skipped.push(format!("event {r}: {reason}")),
        }
        r = r.wrapping_add(1);
    }
    out.read = w;
    out
}

/// [`drain`] for a ring this code owns: drains it and stores the read index.
pub fn drain_mut(ring: &mut [u8]) -> Drained {
    let drained = drain(ring);
    if ring.len() == BYTE_LEN {
        set_word(ring, 1, drained.read);
    }
    drained
}
