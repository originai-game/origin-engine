# og-input

Input as the game sees it (design §2.2, IDL `input` in
`crates/og-api/api.ogt`): the engine-owned `og.Input`, `og.Pointers` and
`og.Gamepad` resources systems poll,
the six input events, the `InputSink` the probe and the command channel
deliver through, and the decoder for the web shell's SharedArrayBuffer input
ring. It depends on `og-core` and `og-ecs` only and reads no clock.

## API map

| Item | Entry points |
|---|---|
| `Input` (`og.Input`, version 2) | `down(code)`, `pressed`, `released`, `axis(neg, pos)`, `axis2`, `button_down(PointerButton)`, `button_pressed`, `button_released`, `contains(KeySet, i)`, `word(KeySet, w)`; `apply(&InjectedInput) -> Option<Vec<InputEvent>>`, `end_tick() -> Vec<InputEvent>` |
| `Pointers` (`og.Pointers`, version 1) | `position(id)`, `start(id)`, `down(id)`, `is_pressed(id)`, `is_released(id)`, `count()`; masks `held`, `pressed`, `released` (bit `id`); `movement` (pointer 0's relative movement this tick); `apply`, `end_tick` |
| `PointerLock` (`og.PointerLock`, version 1) | `wanted` (the game's), `locked` (the host's, from `InjectedInput::PointerLock`) |
| `Gamepad` (`og.Gamepad`, version 1) | `masks(pad) -> (held, pressed, released)`, `is_connected(pad)`, `down(button, pad)`, `pressed`, `released`, `axis(GamepadAxis, pad)`; `GAMEPAD_BUTTONS` (17 standard-mapping names, bit order), `gamepad_button(name)`, `MAX_GAMEPADS = 4`, `MAX_POINTERS = 5` |
| events | `KeyDown { key: Str64, repeat }`, `KeyUp { key }`, `PointerDown { x, y, button, id }`, `PointerUp { x, y, button, id }`, `PointerMove { x, y, id, movementX, movementY }`, `Wheel { deltaX, deltaY }` as `og.KeyDown` ... `og.Wheel`; `InputEvent` (one of them) `.send(world)`; `register(world)` |
| injected input | `InjectedInput::{Key, Pointer { .., id, movement_x, movement_y }, PointerLock { locked }, Wheel, Gamepad(GamepadState { pad, connected, buttons, axes, triggers })}` with `from_json`/`to_json` (protocol 5), `KeyAction`, `PointerAction`, `PointerButton` (`bit`, `index`, `name`, `from_index`, `from_name`), `Modifiers` |
| keys | `KEY_CODES` (106 `KeyboardEvent.code` names, the key-set bit order), `KEY_SET_BITS = 128`, `key_index`, `key_for_char` (US layout) |
| sink | `InputSink { attach, inject, end_tick, accepted }`, `InputLayer` (alias `WorldInput`), `InputLayer::drain_ring(world, ring) -> RingReport` |
| ring | `ring::{drain, drain_mut, decode_slot, encode_slot, write, new_ring}`, `HEADER_WORDS`, `SLOT_WORDS`, `SLOTS`, `KEY_BYTES`, `BYTE_LEN`, `KINDS`, `ACTIONS`, `BUTTONS` |

## The resource

`og.Input` is the state at the start of a tick. Its IDL members are
`pointer` (vec2, CSS px), `pointerInside` (bool, one byte) and `wheel` (vec2,
the scroll accumulated for this tick) plus the methods; the rest is the
state the methods read, laid out for typed access from TypeScript:

| fields | type | meaning |
|---|---|---|
| `buttons`, `buttonsPressed`, `buttonsReleased`, `buttonsReleaseNext` | u32 | masks: left 1, middle 2, right 4 |
| `keys0..3`, `pressed0..3`, `released0..3`, `releaseNext0..3` | u32 | 128-bit key sets over `KEY_CODES`: code `i` is word `i / 32`, bit `i % 32` |

A TypeScript `Input.down("KeyW")` is `keys[i >> 5] >>> (i & 31) & 1` with
`i = KEY_CODES.indexOf("KeyW")`. `KEY_CODES` only ever grows at the end.

## Several pointers and gamepads

`og.Pointers` holds every pointer on the play surface: pointer 0 is
`og.Input`'s pointer (the mouse, the first touch, what protocol 5 means) and
ids 1 to 4 are further touches, which never move `og.Input`'s pointer or
its buttons. Each has a position, the position it went down at, and
held/pressed/released bits with the same one-tick edges as `og.Input`; its
events carry its `id`. The web shell gives each new touch the lowest free
id; the probe's JSON takes an optional `"id"` (0 when absent), and an id
of 5 or more is refused.

`og.Gamepad` holds up to four pads: connected bits, held/pressed/released
button masks over `GAMEPAD_BUTTONS` (A, B, X, Y, LB, RB, LT, RT, Back,
Start, LS, RS, d-pad up/down/left/right, Home), both sticks (y down, as the
Gamepad API reports) and both analog triggers. A pad's whole state arrives
as one `InjectedInput::Gamepad` whenever the shell sees it change (protocol
5 JSON `{ "kind": "gamepad", "pad", "connected", "buttons", "axes",
"triggers" }`, the command channel's `gamepad:` action); edges are
computed against the previous state, so a press and a release between two
ticks leave both edges, and a disconnected pad holds nothing. Pad 4 or more
is refused. Both resources are world state: snapshots carry them and replays
are exact (`tests/pads.rs`). The web shell also keeps mapping pad buttons
to keys (`gamepad.ts`), the fallback for games that read only `og.Input`.

## Mouse-look

A pointer event carries the mouse's relative movement (`movementX`,
`movementY` in protocol 5 JSON, 0 when absent; the browser's, which keeps
coming while the pointer is locked and its position stands still).
`og.Pointers.movement` sums pointer 0's over the events before a tick and
is zero again when the tick ends; a move whose movement is not zero raises
`PointerMove` even at the same position. `og.PointerLock.wanted` is the
game's ask for the lock; the host reports the lock with `{ "kind":
"pointerLock", "locked" }` (ring kind 6), which sets `locked` and nothing
else. `tests/mouselook.rs` holds both, and the ring and JSON round trips.

## Semantics

- **Timing**: an event delivered between ticks N-1 and N is visible to every
  system of tick N: in the resource and as an event (events sent between
  ticks are delivered at the next step). The host calls `end_tick` after the
  step; this tick's edges and wheel expire there.
- **Edges last one tick**: `pressed`/`released` and the button edges are
  set by the event and cleared at the end of the tick that saw them. A down
  and an up between the same two ticks leave both edges set and the key not
  held, so a tap shorter than a tick is seen.
- **Press and click**: `press` holds the key and `click` the button until
  the end of the next tick, which sets their released edge for the tick
  after and raises `KeyUp` / `PointerUp` (at the pointer).
- **Repeats**: a down for a held key raises `KeyDown { repeat: true }` and no
  edge. An up for a key not held does nothing. Button `none` only moves.
- **Modifiers**: a key `down` or `press` whose `modifiers` name a modifier
  that neither of its keys holds (Shift, Control, Alt, `platform` as
  `MetaLeft`: Command, told apart from Control) presses its left key first
  for that tick (held, a pressed edge, `KeyDown`, released when the tick
  ends); a held one stays as it is, and an `up` releases none (they were
  held while it happened).
- **Pointer moves**: `PointerMove` for every `move`, and before the button
  or wheel event of any other event that changes the position.
- **Order**: events reach the world in injection order; the ring is drained
  oldest first, once per frame before the frame's ticks; probe and command
  input is injected where its call runs. The events a tick's end raises
  follow in key-code order, then left, middle, right.
- **Refusals**: a key code outside `KEY_CODES` is refused (`inject` is
  `false`) and changes nothing.
- **World state**: the resource and the event queues are world state, so
  injected input is in snapshots and the world hash and replays exactly; the
  sink holds only the `accepted` count.

`attach` registers `og.Input`, the six events, `og.Pointers` and
`og.Gamepad` and inserts the resources. The events' `key` is a `str64`, `button` an enum
(`none left middle right`).

## The input ring

`packages/engine/src/web/input.ts` writes it on the main thread; the Worker
drains it once per frame. The buffer is `(2 + 256 × 16) × 4` = 16,392 bytes
of little-endian 32-bit words:

| words | content |
|---|---|
| 0 | write index, monotonic (the writer publishes a slot with `Atomics.store`) |
| 1 | read index, monotonic (the reader stores it back) |
| 2 + 16·s | slot `s = index % 256`: `kind` (0 key, 1 pointer, 2 wheel, 3 gamepad), `action` (0 press, 1 down, 2 up, 3 click, 4 move), `button` (0 none, 1 left, 2 middle, 3 right), key length (≤ 32), `x`, `y`, `deltaX`, `deltaY` as f32 bits, then 32 bytes of key code (words 8–15); a pointer slot has its id in word 8 and its movementX, movementY in words 9–10 (f32); kind 6 is a pointer lock report, word 1 locked (1) or released (0) |
| gamepad slot | word 2 the pad, 3 connected, 4–7 the axes (f32), 8 the button mask, 9–10 the triggers (f32) |

The writer drops an event when `write - read ≥ 256`, so the reader never
sees an overwritten slot. Modifiers do not cross the ring. `ring::drain`
takes a copy of the bytes (or the live buffer), returns the events oldest
first, the slots it could not decode with a reason, and the new read index
for word 1. `ring::write` mirrors `writeRing` for native hosts and tests.

## Adopting it in the probe

`InputSink` has og-probe's interim trait shape and `InjectedInput`,
`KeyAction`, `PointerAction`, `PointerButton`, `Modifiers`, `KEY_CODES`,
`key_index` and `key_for_char` are the same items, so the probe's
`input.rs` can become `pub use og_input::{...}` with `WorldInput` (an alias
of `InputLayer`). The resource changes layout (`og.Input` version 2:
`pointerInside` is a bool, key sets are `u32` words); the button masks and
their Rust field names (`buttons`, `buttons_pressed`, `buttons_released`)
are unchanged.

## Tests

`cargo test -p og-input`:

- `tests/edges.rs`: a seven-tick script (hold, release, press, click, two
  wheel steps, a sub-tick tap, a repeat, an axis, an unknown code) observed
  by a game system tick by tick, against the expected table; writes
  `$CARGO_TARGET_TMPDIR/input-edges-transcript.json`. A snapshot between
  ticks restored into a fresh world replays the rest identically. A key
  event's modifiers are held for its tick (Command as `MetaLeft`, not
  Control), a held one stays, an `up` releases none.
- `tests/ring.rs`: `tests/fixtures/ring.bin`, written by the web shell's own
  `writeRing` with the live events across the slot wrap, decodes to what its
  `drainRing` read (`ring.json`); the Rust writer round-trips, a full ring
  refuses, a bad slot is skipped with a reason; the layer drains a ring into
  the world. Regenerate the fixture with
  `node crates/og-input/tests/fixtures/make-ring.mjs` when `input.ts`
  changes (it now has 11 events, a second touch and a pad among them).
- `tests/mouselook.rs`: movement summed over a tick and gone the next, a
  touch's movement not counted, `PointerMove` at a still position, the lock
  report reaching `locked` and leaving `wanted`; movement and lock reports
  across protocol 5 JSON and the ring.
- `tests/pads.rs`: two touches and a pad over a seven-tick script (edges
  for one tick, a sub-tick tap on B, ids on events, og.Input following only
  pointer 0, a click on touch 3, pointer 5 and pad 4 refused, a disconnect
  holding nothing), observed by a game system; writes
  `$CARGO_TARGET_TMPDIR/input-pads-transcript.json`. A snapshot restored
  mid-script replays the rest to the same hash.
