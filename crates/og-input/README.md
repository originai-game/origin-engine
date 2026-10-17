# og-input

Input as the game sees it (design §2.2, IDL `input` in
`crates/og-api/api.ogt`): the engine-owned `og.Input` resource systems poll,
the six input events, the `InputSink` the probe and the command channel
deliver through, and the decoder for the web shell's SharedArrayBuffer input
ring. It depends on `og-core` and `og-ecs` only and reads no clock.

## API map

| Item | Entry points |
|---|---|
| `Input` (`og.Input`, version 2) | `down(code)`, `pressed`, `released`, `axis(neg, pos)`, `axis2`, `button_down(PointerButton)`, `button_pressed`, `button_released`, `contains(KeySet, i)`, `word(KeySet, w)`; `apply(&InjectedInput) -> Option<Vec<InputEvent>>`, `end_tick() -> Vec<InputEvent>` |
| events | `KeyDown { key: Str64, repeat }`, `KeyUp { key }`, `PointerDown { x, y, button }`, `PointerUp { x, y, button }`, `PointerMove { x, y }`, `Wheel { deltaX, deltaY }` as `og.KeyDown` ... `og.Wheel`; `InputEvent` (one of them) `.send(world)`; `register(world)` |
| injected input | `InjectedInput::{Key, Pointer, Wheel}` with `from_json`/`to_json` (protocol 5), `KeyAction`, `PointerAction`, `PointerButton` (`bit`, `index`, `name`, `from_index`, `from_name`), `Modifiers` |
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

`attach` registers, in this order, `og.Input`, `og.KeyDown`, `og.KeyUp`,
`og.PointerDown`, `og.PointerUp`, `og.PointerMove`, `og.Wheel` and inserts
the resource. The events' `key` is a `str64`, `button` an enum
(`none left middle right`).

## The input ring

`packages/engine/src/web/input.ts` writes it on the main thread; the Worker
drains it once per frame. The buffer is `(2 + 256 × 16) × 4` = 16,392 bytes
of little-endian 32-bit words:

| words | content |
|---|---|
| 0 | write index, monotonic (the writer publishes a slot with `Atomics.store`) |
| 1 | read index, monotonic (the reader stores it back) |
| 2 + 16·s | slot `s = index % 256`: `kind` (0 key, 1 pointer, 2 wheel), `action` (0 press, 1 down, 2 up, 3 click, 4 move), `button` (0 none, 1 left, 2 middle, 3 right), key length (≤ 32), `x`, `y`, `deltaX`, `deltaY` as f32 bits, then 32 bytes of key code (words 8–15) |

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
  ticks restored into a fresh world replays the rest identically.
- `tests/ring.rs`: `tests/fixtures/ring.bin`, written by the web shell's own
  `writeRing` with the live events across the slot wrap, decodes to what its
  `drainRing` read (`ring.json`); the Rust writer round-trips, a full ring
  refuses, a bad slot is skipped with a reason; the layer drains a ring into
  the world. Regenerate the fixture with
  `node crates/og-input/tests/fixtures/make-ring.mjs` when `input.ts`
  changes.
