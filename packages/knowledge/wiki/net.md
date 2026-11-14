<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `net` module

Deterministic lockstep over a room (docs/design.md §2.10): every peer runs the whole world, and what crosses the room is each player's input, one per tick, and the world's hash. Turn it on with og.Lockstep and join a room (og.platform.roomJoin); once the room holds `players` peers the authority (the lowest peer number) starts, and tick t runs when every seated player's input for t is in (in rollback, on a prediction re-run when the input differs).

Systems read og.PlayerInput and give this peer's input with og.net.submit; og.Desync reports the first tick whose hashes differ.

See `networking`.

- **TypeScript**: functions are the namespace `og.net`; components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Core** (what most games use): `og.Lockstep`, `og.PlayerInput`, `og.SeatDropped`, `og.SeatRejoined`, `og.net.submit`; 16 more are advanced (`og api show net --all`).
- **Index**: [every module and guide](index.md).

```ogt
module net {
  doc "Deterministic lockstep over a room (docs/design.md §2.10): every peer runs the whole world, and what crosses the room is each player's input, one per tick, and the world's hash. Turn it on with og.Lockstep and join a room (og.platform.roomJoin); once the room holds `players` peers the authority (the lowest peer number) starts, and tick t runs when every seated player's input for t is in (in rollback, on a prediction re-run when the input differs)."
  doc "Systems read og.PlayerInput and give this peer's input with og.net.submit; og.Desync reports the first tick whose hashes differ."
  # declarations follow, each on its own page
```

<sub>Module `net` line 1</sub>

## Contents

| Name | Kind | Signature | Summary |
|---|---|---|---|
| [`NetMode`](net/type-NetMode.md) | enum |  | How a room's peers take their ticks (og.Lockstep.mode). |
| [`NetMode()`](#column-NetMode) | column kind |  | A NetMode field: the variant's index. |
| [`og.Lockstep`](net/record-Lockstep.md) | resource |  | Lockstep's settings and this peer's place. |
| [`og.PlayerInput`](net/record-PlayerInput.md) | component |  | One player's input for this tick, the same on every peer: the engine writes every field but `player` at the start of each tick from the seat's input (zero for a seat nobody holds, and before its hold… |
| [`og.Desync`](net/record-Desync.md) | event |  | This peer's world and another's differ: sent once, at the start of the tick after the hashes showed it. |
| [`og.SeatDropped`](net/record-SeatDropped.md) | event |  | A seat's player was dropped: it left the room, or stalled past og.Lockstep.stallTimeoutMs. |
| [`og.SeatRejoined`](net/record-SeatRejoined.md) | event |  | A seat held before is taken again, by its returning player (og.Lockstep.rejoin) or a newcomer: sent at the start of the first tick of its new holder's input, the same tick on every peer (hashed). |
| [`og.PeerStalled`](net/record-PeerStalled.md) | event |  | A running peer has sent nothing for a second (or half of stallTimeoutMs, if shorter): for a "waiting for player" message. |
| [`InputDesc`](net/type-InputDesc.md) | desc struct |  | This peer's player's input: og.PlayerInput's fields but `player` and `tick`. |
| [`NetShapeDesc`](net/type-NetShapeDesc.md) | desc struct |  | A game's own input shape (og.net.shape): the record each seat's input fills in place of og.PlayerInput, and the commands a seat gives on single ticks (an RTS's orders). |
| [`LockstepSeat`](net/type-LockstepSeat.md) | struct |  | One peer's hold on a seat. |
| [`LockstepStatus`](net/type-LockstepStatus.md) | struct |  | Lockstep on this peer, as the probe and a HUD read it. |
| [`og.net.submit`](net/fn-submit.md) | function | `(world: World, input?: InputDesc): Result<void>` | Give this peer's player's input: in lockstep, for the tick inputDelayTicks ticks from the one running (sent to every peer when the tick ends); otherwise, for the next tick. |
| [`og.net.shape`](net/fn-shape.md) | function | `(world: World, desc?: NetShapeDesc): Result<void>` | Give the room a game's own input shape: a component of the game's in place of og.PlayerInput, and commands (an RTS's orders, a card played, a unit built) that act on one tick only, never repeated by … |
| [`og.net.submitInput`](net/fn-submitInput.md) | function | `<C extends AnyComponent>(world: World, component: C, value: Partial<Value<C>>): Result<void>` | Give this peer's player's input when og.net.shape named an input component: its fields as og.net.submit's (`player` and `tick` are the engine's). |
| [`og.net.command`](net/fn-command.md) | function | `<C extends AnyComponent>(world: World, component: C, value: Partial<Value<C>>): Result<void>` | Give a command for this peer's player: in lockstep, for the tick inputDelayTicks ticks on (with this tick's input), sent once and read on every peer by og.net.commands at that tick; alone, or with lo… |
| [`og.net.commands`](net/fn-commands.md) | function | `<C extends AnyComponent>(world: World, component: C): Result<readonly Value<C>[]>` | The commands every seat gave for the tick now running, the same list on every peer: by seat, then in the order given, each with `player` set to its seat. |
| [`og.net.status`](net/fn-status.md) | function | `(world: World): LockstepStatus` | Lockstep on this peer: running, the seats and who it waits for. |
| [`og.net.resimulating`](net/fn-resimulating.md) | function | `(world: World): boolean` | Whether the tick running is a rollback's re-simulation: the world was rewound and this tick runs again, so its presentation already happened once. |
| [`og.net.finalHash`](net/fn-finalHash.md) | function | `(world: World, tick: number): string \| null` | The shared hash (og.net.hash) of the final world of `tick`, hex: the world every peer agrees on once each input for the ticks before it arrived, kept for the last 1024 final ticks while hashing (hash… |
| [`og.net.hash`](net/fn-hash.md) | function | `(world: World): string` | The world's shared hash now, hex: the hash lockstep compares, which leaves this peer's session out (og.Input, og.Lockstep, input and room events, the UI's focus, ...). |

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

<sub>Module `net` line 13</sub>
