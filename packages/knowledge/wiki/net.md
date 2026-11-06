<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `net` module

Deterministic lockstep over a room (docs/design.md §2.10): every peer runs the whole world, and what crosses the room is each player's input, one per tick, and the world's hash. Turn it on with og.Lockstep (`enabled`, `players`); join a room (og.platform.roomJoin, or `og run --room ws://127.0.0.1:P/<room>` with `og relay` natively). Until the room holds `players` peers the world runs freely on each (a lobby, the local input driving the local player); then the authority (the lowest peer number) starts: every other peer takes its world's snapshot, the seats go to the peers in number order, and from then on tick t runs only when every seated player's input for t is in. Once the room is full, the other peers' worlds hold for that snapshot (for at most stallTimeout, 5 s when it is 0), so a slow authority's start does not find them far into a lobby it replaces. A peer that joins a running room takes a free seat (or watches) from a snapshot a running peer hands it.

Inputs are og.PlayerInput: one entity per player carries it with `player` set to its seat; the engine writes every other field at the start of each tick from that seat's input for the tick. A system reads the inputs from og.PlayerInput, never from og.Input, and gives this peer's own with og.net.submit (from a system that reads og.Input: what the local player holds now, sent for the tick inputDelay ticks on). Without a room, or with lockstep off, og.net.submit drives the local player's og.PlayerInput the next tick, so the same game plays alone.

Rollback (og.Lockstep.mode `rollback`, maxPrediction) keeps the same seats, inputs and hashes but does not wait for a late input: the tick runs on a prediction (the player's last input) and, when the real one differs, the world is rewound to the engine's own snapshot of that tick and runs forward again in the same frame, sounds played once and the view snapped. The peers agree on the confirmed world (og.net.finalHash); og.net.resimulating tells a game's own presentation that a tick runs again. Inputs cross the room unreliably and are repeated until acknowledged, so a lost message costs time, not the room.

A peer that goes silent (a frozen tab, a lost connection the room has not noticed yet) is reported as og.PeerStalled after a second, and dropped after og.Lockstep.stallTimeout: the authority (the lowest running peer that is not itself silent) agrees with every other running peer how far the seat's inputs reach, and every peer empties the seat from the tick after its last input, so og.SeatDropped arrives at one tick everywhere and the room plays on; a peer that leaves the room is dropped the same way at once. A dropped seat's input is empty (og.PlayerInput's zero value), so a held move or fire stops; a game that wants a bot there reads og.SeatDropped and drives that seat itself. With og.Lockstep.rejoin a dropped player that comes back (the same peer, or a newcomer of the same name) takes its seat again from the authority's world, og.SeatRejoined at one tick on every peer. Wall time decides only when a peer is reported and dropped, never what the confirmed world is.

Every tick each peer hashes its world without its own session (og.Input, og.Pointers, og.Gamepad, og.PointerLock, og.Audio, og.Camera, og.Lockstep, the gameplay actions of og.input, the input, room, platform, asset and stream events, the UI's focus and hover) and every hashEvery ticks sends the hashes since its last batch; the first tick whose hashes differ is reported once as og.Desync, and the probe's `desync` dump (`dump_desync`) names it with the component-level diff of the two peers' snapshots of that tick: the answer to what is not deterministic (Math.random, Date.now, a wall clock, iteration over a Set of objects, a module variable, input read from og.Input or og.input's actions).

- **TypeScript**: functions are the namespace `og.net`; components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Index**: [every module and guide](index.md).

```ogt
module net {
  doc "Deterministic lockstep over a room (docs/design.md §2.10): every peer runs the whole world, and what crosses the room is each player's input, one per tick, and the world's hash. Turn it on with og.Lockstep (`enabled`, `players`); join a room (og.platform.roomJoin, or `og run --room ws://127.0.0.1:P/<room>` with `og relay` natively). Until the room holds `players` peers the world runs freely on each (a lobby, the local input driving the local player); then the authority (the lowest peer number) starts: every other peer takes its world's snapshot, the seats go to the peers in number order, and from then on tick t runs only when every seated player's input for t is in. Once the room is full, the other peers' worlds hold for that snapshot (for at most stallTimeout, 5 s when it is 0), so a slow authority's start does not find them far into a lobby it replaces. A peer that joins a running room takes a free seat (or watches) from a snapshot a running peer hands it."
  doc "Inputs are og.PlayerInput: one entity per player carries it with `player` set to its seat; the engine writes every other field at the start of each tick from that seat's input for the tick. A system reads the inputs from og.PlayerInput, never from og.Input, and gives this peer's own with og.net.submit (from a system that reads og.Input: what the local player holds now, sent for the tick inputDelay ticks on). Without a room, or with lockstep off, og.net.submit drives the local player's og.PlayerInput the next tick, so the same game plays alone."
  doc "Rollback (og.Lockstep.mode `rollback`, maxPrediction) keeps the same seats, inputs and hashes but does not wait for a late input: the tick runs on a prediction (the player's last input) and, when the real one differs, the world is rewound to the engine's own snapshot of that tick and runs forward again in the same frame, sounds played once and the view snapped. The peers agree on the confirmed world (og.net.finalHash); og.net.resimulating tells a game's own presentation that a tick runs again. Inputs cross the room unreliably and are repeated until acknowledged, so a lost message costs time, not the room."
  doc "A peer that goes silent (a frozen tab, a lost connection the room has not noticed yet) is reported as og.PeerStalled after a second, and dropped after og.Lockstep.stallTimeout: the authority (the lowest running peer that is not itself silent) agrees with every other running peer how far the seat's inputs reach, and every peer empties the seat from the tick after its last input, so og.SeatDropped arrives at one tick everywhere and the room plays on; a peer that leaves the room is dropped the same way at once. A dropped seat's input is empty (og.PlayerInput's zero value), so a held move or fire stops; a game that wants a bot there reads og.SeatDropped and drives that seat itself. With og.Lockstep.rejoin a dropped player that comes back (the same peer, or a newcomer of the same name) takes its seat again from the authority's world, og.SeatRejoined at one tick on every peer. Wall time decides only when a peer is reported and dropped, never what the confirmed world is."
  doc "Every tick each peer hashes its world without its own session (og.Input, og.Pointers, og.Gamepad, og.PointerLock, og.Audio, og.Camera, og.Lockstep, the gameplay actions of og.input, the input, room, platform, asset and stream events, the UI's focus and hover) and every hashEvery ticks sends the hashes since its last batch; the first tick whose hashes differ is reported once as og.Desync, and the probe's `desync` dump (`dump_desync`) names it with the component-level diff of the two peers' snapshots of that tick: the answer to what is not deterministic (Math.random, Date.now, a wall clock, iteration over a Set of objects, a module variable, input read from og.Input or og.input's actions)."
  # declarations follow, each on its own page
```

<sub>`api.ogt` line 26128</sub>

## Contents

| Name | Kind | Signature | Summary |
|---|---|---|---|
| [`NetMode`](net/type-NetMode.md) | enum |  | How a room's peers take their ticks (og.Lockstep.mode). |
| [`NetMode()`](#column-NetMode) | column kind |  | A NetMode field: the variant's index. |
| [`og.Lockstep`](net/record-Lockstep.md) | resource |  | Lockstep's settings and this peer's place. |
| [`og.PlayerInput`](net/record-PlayerInput.md) | component |  | One player's input for this tick, the same on every peer: the engine writes every field but `player` at the start of each tick from the seat's input (zero for a seat nobody holds, and before its hold… |
| [`og.Desync`](net/record-Desync.md) | event |  | This peer's world and another's differ: sent once, at the start of the tick after the hashes showed it. |
| [`og.SeatDropped`](net/record-SeatDropped.md) | event |  | A seat's player was dropped: it left the room, or stalled past og.Lockstep.stallTimeout. |
| [`og.SeatRejoined`](net/record-SeatRejoined.md) | event |  | A seat held before is taken again, by its returning player (og.Lockstep.rejoin) or a newcomer: sent at the start of the first tick of its new holder's input, the same tick on every peer (hashed). |
| [`og.PeerStalled`](net/record-PeerStalled.md) | event |  | A running peer has sent nothing for a second (or half of stallTimeout, if shorter): for a "waiting for player" message. |
| [`InputDesc`](net/type-InputDesc.md) | desc struct |  | This peer's player's input: og.PlayerInput's fields but `player` and `tick`. |
| [`NetShapeDesc`](net/type-NetShapeDesc.md) | desc struct |  | A game's own input shape (og.net.shape): the record each seat's input fills in place of og.PlayerInput, and the commands a seat gives on single ticks (an RTS's orders). |
| [`LockstepSeat`](net/type-LockstepSeat.md) | struct |  | One peer's hold on a seat. |
| [`LockstepStatus`](net/type-LockstepStatus.md) | struct |  | Lockstep on this peer, as the probe and a HUD read it. |
| [`og.net.submit`](net/fn-submit.md) | function | `(world: World, input?: InputDesc): Result<void>` | Give this peer's player's input: in lockstep, for the tick inputDelay ticks from the one running (sent to every peer when the tick ends); otherwise, for the next tick. |
| [`og.net.shape`](net/fn-shape.md) | function | `(world: World, desc?: NetShapeDesc): Result<void>` | Give the room a game's own input shape: a component of the game's in place of og.PlayerInput, and commands (an RTS's orders, a card played, a unit built) that act on one tick only, never repeated by … |
| [`og.net.submitInput`](net/fn-submitInput.md) | function | `<C extends AnyComponent>(world: World, component: C, value: Partial<Value<C>>): Result<void>` | Give this peer's player's input when og.net.shape named an input component: its fields as og.net.submit's (`player` and `tick` are the engine's). |
| [`og.net.command`](net/fn-command.md) | function | `<C extends AnyComponent>(world: World, component: C, value: Partial<Value<C>>): Result<void>` | Give a command for this peer's player: in lockstep, for the tick inputDelay ticks on (with this tick's input), sent once and read on every peer by og.net.commands at that tick; alone, or with lockste… |
| [`og.net.commands`](net/fn-commands.md) | function | `<C extends AnyComponent>(world: World, component: C): Result<readonly Value<C>[]>` | The commands every seat gave for the tick now running, the same list on every peer: by seat, then in the order given, each with `player` set to its seat. |
| [`og.net.status`](net/fn-status.md) | function | `(world: World): LockstepStatus` | Lockstep on this peer: running, the seats and who it waits for. |
| [`og.net.resimulating`](net/fn-resimulating.md) | function | `(world: World): boolean` | Whether the tick running is a rollback's re-simulation: the world was rewound and this tick runs again, so its presentation already happened once. |
| [`og.net.finalHash`](net/fn-finalHash.md) | function | `(world: World, tick: number): string \| null` | The shared hash (og.net.hash) of the final world of `tick`, hex: the world every peer agrees on once each input for the ticks before it arrived, kept for the last 1024 final ticks while hashing (hash… |
| [`og.net.hash`](net/fn-hash.md) | function | `(world: World): string` | The world's shared hash now, hex: the hash lockstep compares, which leaves this peer's session out (og.Input, og.Lockstep, input and room events, the UI's focus; module docs). |

## Column kinds

Each is a factory, `NetMode(value?, options?)`, that makes one field of a component or resource layout.

| Column | Value | Typed array | Stride | Doc |
|---|---|---|---|---|
| <a id="column-NetMode"></a>`NetMode` | `NetMode` | `Uint32Array` | 1 | A NetMode field: the variant's index. |

```ogt
column NetMode {
  doc "A NetMode field: the variant's index."
  array Uint32Array
  stride 1
  value NetMode
}
```

<sub>`api.ogt` line 26141</sub>
