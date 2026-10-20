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
//! One slot is 16 words: `kind` (0 key, 1 pointer, 2 wheel, 3 gamepad),
//! `action` (0 press, 1 down, 2 up, 3 click, 4 move), `button` (0 none, 1
//! left, 2 middle, 3 right), the key's UTF-8 length (≤ 32), then `x`, `y`,
//! `deltaX`, `deltaY` as f32 bits, then 32 bytes of the key code (words
//! 8–15); a pointer slot carries its pointer id (0 to 4) in word 8 instead.
//! A gamepad slot is the pad's whole state: word 2 the pad, word 3
//! connected (0 or 1), words 4–7 the axes (left x, y, right x, y), word 8
//! the button mask, words 9–10 the triggers. A text slot (kind 4, an
//! engine extension) carries word 1 0 for committed text or 1 for an input
//! method's composition, word 2 1 when it continues the text of the slot
//! before it (a text longer than [`TEXT_BYTES`] is split between characters
//! over several slots, which [`drain`] joins), word 3 the UTF-8 length and
//! the text in words 4–15. A key or pointer slot carries
//! the event's time in words 6–7 (an f64: the page's
//! `performance.timeOrigin + timeStamp`, ms; 0 unknown), which the Worker
//! turns into the event's `at` (IDL `KeyInput.at`) before it injects it;
//! [`decode_slot`] leaves it out. Unused words are 0. The writer refuses an event when
//! `write - read ≥ 256` (the ring is full; the event is dropped), so a
//! reader never sees an overwritten slot. Modifiers do not cross the ring.
//!
//! Indices are compared with wrapping `u32` arithmetic here; the TypeScript
//! writer indexes slots with `%` on an `i32`, which stays correct for the
//! first 2^31 events of a page.

use crate::event::{
    GamepadState, InjectedInput, KeyAction, MAX_GAMEPADS, Modifiers, PointerAction, PointerButton,
};

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
/// Bytes of text a text slot holds (words 4–15).
pub const TEXT_BYTES: usize = 48;
/// `kind` values, by index.
pub const KINDS: [&str; 5] = ["key", "pointer", "wheel", "gamepad", "text"];
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
    if w(0) == 4 {
        let len = w(3) as usize;
        if len > TEXT_BYTES {
            return Err(format!("text length {}", w(3) as i32));
        }
        let text = std::str::from_utf8(&slot[16..16 + len])
            .map_err(|_| "text that is not UTF-8".to_owned())?;
        return match w(1) {
            0 | 1 => Ok(InjectedInput::Text {
                text: text.to_owned(),
                composing: w(1) == 1,
            }),
            a => Err(format!("text action {}", a as i32)),
        };
    }
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
                id: w(8),
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
        3 => {
            let state = GamepadState {
                pad: w(2),
                connected: w(3) != 0,
                buttons: w(8),
                axes: [f(4), f(5), f(6), f(7)],
                triggers: [f(9), f(10)],
            };
            if state.pad as usize >= MAX_GAMEPADS {
                return Err(format!("pad {}", w(2) as i32));
            }
            if !state
                .axes
                .iter()
                .chain(&state.triggers)
                .all(|v| v.is_finite())
            {
                return Err("a non-finite gamepad value".into());
            }
            Ok(InjectedInput::Gamepad(state))
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
    if let InjectedInput::Text { text, composing } = event {
        let mut cut = text.len().min(TEXT_BYTES);
        while !text.is_char_boundary(cut) {
            cut -= 1;
        }
        set_word(&mut slot, 0, 4);
        set_word(&mut slot, 1, u32::from(*composing));
        set_word(&mut slot, 3, cut as u32);
        slot[16..16 + cut].copy_from_slice(&text.as_bytes()[..cut]);
        return slot;
    }
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
            id,
        } => {
            set_word(&mut slot, 8, *id);
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
        InjectedInput::Gamepad(g) => {
            set_word(&mut slot, 3, g.connected as u32);
            set_word(&mut slot, 8, g.buttons);
            set_word(&mut slot, 9, g.triggers[0].to_bits());
            set_word(&mut slot, 10, g.triggers[1].to_bits());
            (3, 0, g.pad, (g.axes[0], g.axes[1]), (g.axes[2], g.axes[3]))
        }
        InjectedInput::Text { .. } => unreachable!("encoded above"),
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
    // A text longer than a slot takes several, the later ones continuing it.
    let slots: Vec<[u8; SLOT_WORDS * 4]> = match event {
        InjectedInput::Text { text, composing } => {
            let mut out = Vec::new();
            let mut rest = text.as_str();
            loop {
                let mut cut = rest.len().min(TEXT_BYTES);
                while !rest.is_char_boundary(cut) {
                    cut -= 1;
                }
                let (head, tail) = rest.split_at(cut);
                let mut slot = encode_slot(&InjectedInput::Text {
                    text: head.to_owned(),
                    composing: *composing,
                });
                set_word(&mut slot, 2, u32::from(!out.is_empty()));
                out.push(slot);
                if tail.is_empty() {
                    break out;
                }
                rest = tail;
            }
        }
        e => vec![encode_slot(e)],
    };
    let (w, r) = (word(ring, 0), word(ring, 1));
    if w.wrapping_sub(r) as usize + slots.len() > SLOTS {
        return false;
    }
    for (k, slot) in slots.iter().enumerate() {
        let at = (HEADER_WORDS + (w.wrapping_add(k as u32) as usize % SLOTS) * SLOT_WORDS) * 4;
        ring[at..at + SLOT_WORDS * 4].copy_from_slice(slot);
    }
    set_word(ring, 0, w.wrapping_add(slots.len() as u32));
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
        let slot = &ring[at..at + SLOT_WORDS * 4];
        match decode_slot(slot) {
            // A text slot that continues the one before joins it.
            Ok(InjectedInput::Text { text, composing })
                if word(slot, 2) == 1
                    && matches!(out.events.last(), Some(InjectedInput::Text { composing: c, .. }) if *c == composing) =>
            {
                if let Some(InjectedInput::Text { text: before, .. }) = out.events.last_mut() {
                    before.push_str(&text);
                }
            }
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
