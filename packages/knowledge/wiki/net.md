<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `net` module

Deterministic lockstep over a room (docs/design.md §2.10): every peer runs the whole world, and what crosses the room is each player's input, one per tick, and the world's hash. Turn it on with og.Lockstep (`enabled`, `players`); join a room (og.platform.roomJoin, or `og run --room ws://127.0.0.1:P/<room>` with `og relay` natively). Until the room holds `players` peers the world runs freely on each (a lobby, the local input driving the local player); then the authority (the lowest peer number) starts: every other peer takes its world's snapshot, the seats go to the peers in number order, and from then on tick t runs only when every seated player's input for t is in. Once the room is full, the other peers' worlds hold for that snapshot (for at most stallTimeout, 5 s when it is 0), so a slow authority's start does not find them far into a lobby it replaces. A peer that joins a running room takes a free seat (or watches) from a snapshot a running peer hands it.

Inputs are og.PlayerInput: one entity per player carries it with `player` set to its seat; the engine writes every other field at the start of each tick from that seat's input for the tick. A system reads the inputs from og.PlayerInput, never from og.Input, and gives this peer's own with og.net.submit (from a system that reads og.Input: what the local player holds now, sent for the tick inputDelay ticks on). Without a room, or with lockstep off, og.net.submit drives the local player's og.PlayerInput the next tick, so the same game plays alone.

Rollback (og.Lockstep.mode `rollback`, maxPrediction) keeps the same seats, inputs and hashes but does not wait for a late input: the tick runs on a prediction (the player's last input) and, when the real one differs, the world is rewound to the engine's own snapshot of that tick and runs forward again in the same frame, sounds played once and the view snapped. The peers agree on the confirmed world (og.net.finalHash); og.net.resimulating tells a game's own presentation that a tick runs again. Inputs cross the room unreliably and are repeated until acknowledged, so a lost message costs time, not the room.

A peer that goes silent (a frozen tab, a lost connection the room has not noticed yet) is reported as og.PeerStalled after a second, and dropped after og.Lockstep.stallTimeout: the authority (the lowest running peer that is not itself silent) agrees with every other running peer how far the seat's inputs reach, and every peer empties the seat from the tick after its last input, so og.SeatDropped arrives at one tick everywhere and the room plays on; a peer that leaves the room is dropped the same way at once. A dropped seat's input is empty (og.PlayerInput's zero value), so a held move or fire stops; a game that wants a bot there reads og.SeatDropped and drives that seat itself. With og.Lockstep.rejoin a dropped player that comes back (the same peer, or a newcomer of the same name) takes its seat again from the authority's world, og.SeatRejoined at one tick on every peer. Wall time decides only when a peer is reported and dropped, never what the confirmed world is.

Every tick each peer hashes its world without its own session (og.Input, og.Pointers, og.Gamepad, og.PointerLock, og.Audio, og.Camera, og.Lockstep, the gameplay actions of og.input, the input, room, platform, asset and stream events, the UI's focus and hover) and every hashEvery ticks sends the hashes since its last batch; the first tick whose hashes differ is reported once as og.Desync, and the probe's `desync` dump (`dump_desync`) names it with the component-level diff of the two peers' snapshots of that tick: the answer to what is not deterministic (Math.random, Date.now, a wall clock, iteration over a Set of objects, a module variable, input read from og.Input or og.input's actions).

- **TypeScript**: functions are the namespace `og.net`; components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Skill**: `og-net`, which teaches the module (how much of it to use, and how to prove it).
- **Index**: [every module and name](index.md).

```ogt
module net {
  # declarations follow, each quoted below
```

<sub>`api.ogt` line 29299</sub>

## Contents

| Name | Kind | Summary |
|---|---|---|
| [`NetMode`](#type-NetMode) | enum | How a room's peers take their ticks (og.Lockstep.mode). |
| [`NetMode()`](#column-NetMode) | column kind | A NetMode field: the variant's index. |
| [`og.Lockstep`](#record-Lockstep) | resource | Lockstep's settings and this peer's place. The game sets enabled, players, inputDelay and hashEvery (the authority's are the room's once it starts: changes while running wait for the next start); the engine sets localPlayer and running. It is this peer's own: never hashed, never compared. |
| [`og.PlayerInput`](#record-PlayerInput) | component | One player's input for this tick, the same on every peer: the engine writes every field but `player` at the start of each tick from the seat's input (zero for a seat nobody holds, and before its holder's first input). Its packed bytes are what crosses the room, read through this layout. |
| [`og.Desync`](#record-Desync) | event | This peer's world and another's differ: sent once, at the start of the tick after the hashes showed it. The probe's `desync` dump (`dump_desync`) names the component and field that differ, once the other peer's snapshot of that tick arrived. |
| [`og.SeatDropped`](#record-SeatDropped) | event | A seat's player was dropped: it left the room, or stalled past og.Lockstep.stallTimeout. Sent at the start of the first tick the seat is empty, the same tick on every peer (a hashed event: the confirmed world holds it); from then on its og.PlayerInput is the zero value until og.SeatRejoined. |
| [`og.SeatRejoined`](#record-SeatRejoined) | event | A seat held before is taken again, by its returning player (og.Lockstep.rejoin) or a newcomer: sent at the start of the first tick of its new holder's input, the same tick on every peer (hashed). |
| [`og.PeerStalled`](#record-PeerStalled) | event | A running peer has sent nothing for a second (or half of stallTimeout, if shorter): for a "waiting for player" message. This peer's own (wall time decides it): never hashed, and a peer that speaks again is reported again the next time it falls silent. |
| [`InputDesc`](#type-InputDesc) | desc struct | This peer's player's input: og.PlayerInput's fields but `player` and `tick`. |
| [`NetShapeDesc`](#type-NetShapeDesc) | desc struct | A game's own input shape (og.net.shape): the record each seat's input fills in place of og.PlayerInput, and the commands a seat gives on single ticks (an RTS's orders). |
| [`LockstepSeat`](#type-LockstepSeat) | struct | One peer's hold on a seat. |
| [`LockstepStatus`](#type-LockstepStatus) | struct | Lockstep on this peer, as the probe and a HUD read it. |
| [`og.net.submit`](#fn-submit) | function | Give this peer's player's input: in lockstep, for the tick inputDelay ticks from the one running (sent to every peer when the tick ends); otherwise, for the next tick. It holds until the next call, so a game calls it once a tick from a system that reads og.Input (or og.Gamepad, or the UI's touch controls). Callable from a system. |
| [`og.net.shape`](#fn-shape) | function | Give the room a game's own input shape: a component of the game's in place of og.PlayerInput, and commands (an RTS's orders, a card played, a unit built) that act on one tick only, never repeated by a prediction. Call it in setup or start, outside a system and before the room starts, the same on every peer: a peer whose shape differs is refused the room's start. Without a call, og.PlayerInput and no commands. |
| [`og.net.submitInput`](#fn-submitInput) | function | Give this peer's player's input when og.net.shape named an input component: its fields as og.net.submit's (`player` and `tick` are the engine's). It holds until the next call. Callable from a system. |
| [`og.net.command`](#fn-command) | function | Give a command for this peer's player: in lockstep, for the tick inputDelay ticks on (with this tick's input), sent once and read on every peer by og.net.commands at that tick; alone, or with lockstep off, at the next tick. Given while a rollback re-simulates a tick it is ignored (its first run gave it already). Callable from a system. |
| [`og.net.commands`](#fn-commands) | function | The commands every seat gave for the tick now running, the same list on every peer: by seat, then in the order given, each with `player` set to its seat. Empty without a command shape. Callable from a system. |
| [`og.net.status`](#fn-status) | function | Lockstep on this peer: running, the seats and who it waits for. |
| [`og.net.resimulating`](#fn-resimulating) | function | Whether the tick running is a rollback's re-simulation: the world was rewound and this tick runs again, so its presentation already happened once. The engine already plays each sound once and snaps the view; a game that triggers its own presentation from a system (a screen shake, a haptic) checks this first. False in lockstep and alone. Callable from a system. |
| [`og.net.finalHash`](#fn-finalHash) | function | The shared hash (og.net.hash) of the final world of `tick`, hex: the world every peer agrees on once each input for the ticks before it arrived, kept for the last 1024 final ticks while hashing (hashEvery above 0). In lockstep every tick run is final; in rollback a tick is final some ticks after it first ran. Null before the tick is final, after it is no longer kept, or outside lockstep. |
| [`og.net.hash`](#fn-hash) | function | The world's shared hash now, hex: the hash lockstep compares, which leaves this peer's session out (og.Input, og.Lockstep, input and room events, the UI's focus; module docs). Two peers at the same tick agree on it exactly when their simulations agree; the world dump's `hash` covers everything and differs between peers. |

## Functions

<a id="fn-submit"></a>

### `og.net.submit`

Give this peer's player's input: in lockstep, for the tick inputDelay ticks from the one running (sent to every peer when the tick ends); otherwise, for the next tick. It holds until the next call, so a game calls it once a tick from a system that reads og.Input (or og.Gamepad, or the UI's touch controls). Callable from a system.

```ts
og.net.submit(world: World, input?: InputDesc): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `input` | `InputDesc` | yes | The buttons and axes. |

**Returns** `Result<void>`: Refused for an axis that is not finite.

Binding symbol `og_net_submit` · publish builds: kept

```ogt
fn submit {
  doc "Give this peer's player's input: in lockstep, for the tick inputDelay ticks from the one running (sent to every peer when the tick ends); otherwise, for the next tick. It holds until the next call, so a game calls it once a tick from a system that reads og.Input (or og.Gamepad, or the UI's touch controls). Callable from a system."
  param world World "The world."
  param input InputDesc "The buttons and axes."
  returns Result.Void "Refused for an axis that is not finite."
}
```

<sub>`api.ogt` line 29490</sub>

<a id="fn-shape"></a>

### `og.net.shape`

Give the room a game's own input shape: a component of the game's in place of og.PlayerInput, and commands (an RTS's orders, a card played, a unit built) that act on one tick only, never repeated by a prediction. Call it in setup or start, outside a system and before the room starts, the same on every peer: a peer whose shape differs is refused the room's start. Without a call, og.PlayerInput and no commands.

```ts
og.net.shape(world: World, desc?: NetShapeDesc): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `desc` | `NetShapeDesc` | yes | The input component, the command component and the most commands per tick. |

**Returns** `Result<void>`: Refused inside a system, while lockstep runs, or for a component that is not a game's own with a u32 `player` field.

Binding symbol `og_net_shape` · publish builds: kept

```ogt
fn shape {
  doc "Give the room a game's own input shape: a component of the game's in place of og.PlayerInput, and commands (an RTS's orders, a card played, a unit built) that act on one tick only, never repeated by a prediction. Call it in setup or start, outside a system and before the room starts, the same on every peer: a peer whose shape differs is refused the room's start. Without a call, og.PlayerInput and no commands."
  param world World "The world."
  param desc NetShapeDesc "The input component, the command component and the most commands per tick."
  returns Result.Void "Refused inside a system, while lockstep runs, or for a component that is not a game's own with a u32 `player` field."
}
```

<sub>`api.ogt` line 29497</sub>

<a id="fn-submitInput"></a>

### `og.net.submitInput`

Give this peer's player's input when og.net.shape named an input component: its fields as og.net.submit's (`player` and `tick` are the engine's). It holds until the next call. Callable from a system.

```ts
og.net.submitInput<C extends AnyComponent>(world: World, component: C, value: Partial<Value<C>>): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `component` | `C` | no | The input component og.net.shape named. |
| `value` | `Partial<Value<C>>` | no | Its fields; omitted ones take their defaults. |

**Returns** `Result<void>`: Refused for another component than the shape's, or a field that does not fit.

Binding symbol `og_net_submit_input` · publish builds: kept

```ogt
fn submitInput {
  doc "Give this peer's player's input when og.net.shape named an input component: its fields as og.net.submit's (`player` and `tick` are the engine's). It holds until the next call. Callable from a system."
  generic C AnyComponent
  param world World "The world."
  param component C "The input component og.net.shape named."
  param value Partial.Value.C "Its fields; omitted ones take their defaults."
  returns Result.Void "Refused for another component than the shape's, or a field that does not fit."
}
```

<sub>`api.ogt` line 29504</sub>

<a id="fn-command"></a>

### `og.net.command`

Give a command for this peer's player: in lockstep, for the tick inputDelay ticks on (with this tick's input), sent once and read on every peer by og.net.commands at that tick; alone, or with lockstep off, at the next tick. Given while a rollback re-simulates a tick it is ignored (its first run gave it already). Callable from a system.

```ts
og.net.command<C extends AnyComponent>(world: World, component: C, value: Partial<Value<C>>): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `component` | `C` | no | The command component og.net.shape named. |
| `value` | `Partial<Value<C>>` | no | Its fields (`player` is the engine's). |

**Returns** `Result<void>`: Refused with no command shape, for another component, or past maxCommands this tick.

Binding symbol `og_net_command` · publish builds: kept

```ogt
fn command {
  doc "Give a command for this peer's player: in lockstep, for the tick inputDelay ticks on (with this tick's input), sent once and read on every peer by og.net.commands at that tick; alone, or with lockstep off, at the next tick. Given while a rollback re-simulates a tick it is ignored (its first run gave it already). Callable from a system."
  generic C AnyComponent
  param world World "The world."
  param component C "The command component og.net.shape named."
  param value Partial.Value.C "Its fields (`player` is the engine's)."
  returns Result.Void "Refused with no command shape, for another component, or past maxCommands this tick."
}
```

<sub>`api.ogt` line 29513</sub>

<a id="fn-commands"></a>

### `og.net.commands`

The commands every seat gave for the tick now running, the same list on every peer: by seat, then in the order given, each with `player` set to its seat. Empty without a command shape. Callable from a system.

```ts
og.net.commands<C extends AnyComponent>(world: World, component: C): Result<readonly Value<C>[]>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `component` | `C` | no | The command component og.net.shape named. |

**Returns** `Result<readonly Value<C>[]>`: Refused for another component than the shape's.

Binding symbol `og_net_commands` · publish builds: kept

```ogt
fn commands {
  doc "The commands every seat gave for the tick now running, the same list on every peer: by seat, then in the order given, each with `player` set to its seat. Empty without a command shape. Callable from a system."
  generic C AnyComponent
  param world World "The world."
  param component C "The command component og.net.shape named."
  returns Result.List.Value.C "Refused for another component than the shape's."
}
```

<sub>`api.ogt` line 29522</sub>

<a id="fn-status"></a>

### `og.net.status`

Lockstep on this peer: running, the seats and who it waits for.

```ts
og.net.status(world: World): LockstepStatus
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |

**Returns** `LockstepStatus`.

Binding symbol `og_net_status` · publish builds: kept

```ogt
fn status {
  doc "Lockstep on this peer: running, the seats and who it waits for."
  param world World "The world."
  returns LockstepStatus
}
```

<sub>`api.ogt` line 29530</sub>

<a id="fn-resimulating"></a>

### `og.net.resimulating`

Whether the tick running is a rollback's re-simulation: the world was rewound and this tick runs again, so its presentation already happened once. The engine already plays each sound once and snaps the view; a game that triggers its own presentation from a system (a screen shake, a haptic) checks this first. False in lockstep and alone. Callable from a system.

```ts
og.net.resimulating(world: World): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |

**Returns** `boolean`.

Binding symbol `og_net_resimulating` · publish builds: kept

```ogt
fn resimulating {
  doc "Whether the tick running is a rollback's re-simulation: the world was rewound and this tick runs again, so its presentation already happened once. The engine already plays each sound once and snaps the view; a game that triggers its own presentation from a system (a screen shake, a haptic) checks this first. False in lockstep and alone. Callable from a system."
  param world World "The world."
  returns bool
}
```

<sub>`api.ogt` line 29536</sub>

<a id="fn-finalHash"></a>

### `og.net.finalHash`

The shared hash (og.net.hash) of the final world of `tick`, hex: the world every peer agrees on once each input for the ticks before it arrived, kept for the last 1024 final ticks while hashing (hashEvery above 0). In lockstep every tick run is final; in rollback a tick is final some ticks after it first ran. Null before the tick is final, after it is no longer kept, or outside lockstep.

```ts
og.net.finalHash(world: World, tick: number): string | null
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `tick` | `number` | no | The tick, in ticks: the world after tick - 1 ran, as og.Time.tick reads. |

**Returns** `string | null`.

Binding symbol `og_net_final_hash` · publish builds: kept

```ogt
fn finalHash {
  doc "The shared hash (og.net.hash) of the final world of `tick`, hex: the world every peer agrees on once each input for the ticks before it arrived, kept for the last 1024 final ticks while hashing (hashEvery above 0). In lockstep every tick run is final; in rollback a tick is final some ticks after it first ran. Null before the tick is final, after it is no longer kept, or outside lockstep."
  param world World "The world."
  param tick f64 "The tick, in ticks: the world after tick - 1 ran, as og.Time.tick reads."
  returns Option.String
}
```

<sub>`api.ogt` line 29542</sub>

<a id="fn-hash"></a>

### `og.net.hash`

The world's shared hash now, hex: the hash lockstep compares, which leaves this peer's session out (og.Input, og.Lockstep, input and room events, the UI's focus; module docs). Two peers at the same tick agree on it exactly when their simulations agree; the world dump's `hash` covers everything and differs between peers.

```ts
og.net.hash(world: World): string
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |

**Returns** `string`.

Binding symbol `og_net_hash` · publish builds: kept

```ogt
fn hash {
  doc "The world's shared hash now, hex: the hash lockstep compares, which leaves this peer's session out (og.Input, og.Lockstep, input and room events, the UI's focus; module docs). Two peers at the same tick agree on it exactly when their simulations agree; the world dump's `hash` covers everything and differs between peers."
  param world World "The world."
  returns String
}
```

<sub>`api.ogt` line 29549</sub>

## Components, resources and events

<a id="record-Lockstep"></a>

### `og.Lockstep` (resource)

Lockstep's settings and this peer's place. The game sets enabled, players, inputDelay and hashEvery (the authority's are the room's once it starts: changes while running wait for the next start); the engine sets localPlayer and running. It is this peer's own: never hashed, never compared.

Token `og.Lockstep`, registered under the schema name `og.Lockstep`; its value type is `Lockstep`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `enabled` | `boolean` | `false` | — | — | — | Whether the world takes its ticks in lockstep when in a room. |
| `players` | `number` | `2` | 1 … 16 | — | — | The seats: the peers lockstep waits for before it starts, and whose inputs each tick waits for. |
| `localPlayer` | `number` | `0` | — | — | read only | This peer's seat, the og.PlayerInput.player its input drives; `players` while it watches (a peer beyond the seats). 0 before the start. |
| `inputDelay` | `number` | `2` | 1 … 30 | `ticks` | — | Ticks between a player's input and the tick it acts on: the time the input has to cross the room. 2 (33 ms at 60 Hz) on a LAN, 4 to 6 over the internet. |
| `hashEvery` | `number` | `10` | 0 … 600 | `ticks` | — | How often the peers exchange their world hashes, in ticks (every tick is hashed, so the first divergent tick is exact); 0 compares nothing. |
| `running` | `boolean` | `false` | — | — | read only | Whether lockstep runs: the room filled the seats and the authority started (until this peer leaves the room or turns it off). |
| `mode` | `NetMode` | `"lockstep"` | — | — | — | lockstep (the default: every tick waits for every input) or rollback (ticks run on predicted inputs and are rewound when an input differs; set before the room starts, the authority's is the room's). |
| `stallTimeout` | `number` | `5000` | 0 … 600000 | `ms` | — | Milliseconds a running peer may stay silent before its seat is dropped (og.SeatDropped); 0 never drops a peer still in the room, so the room waits for it as long as it takes. The authority's is the room's. |
| `rejoin` | `boolean` | `true` | — | — | — | Whether a dropped player that comes back (or a newcomer of the same name) takes its seat again (og.SeatRejoined); when off, a seat once dropped stays empty and newcomers take only seats never held. The authority's is the room's. |
| `maxPrediction` | `number` | `8` | 1 … 30 | `ticks` | — | In rollback, the most ticks this peer runs past the last tick whose inputs are all in before it waits (a peer also predicts no further than it can re-simulate in half a frame). 8 (133 ms at 60 Hz) covers most internet latency on top of inputDelay. |

```ogt
resource Lockstep {
  doc "Lockstep's settings and this peer's place. The game sets enabled, players, inputDelay and hashEvery (the authority's are the room's once it starts: changes while running wait for the next start); the engine sets localPlayer and running. It is this peer's own: never hashed, never compared."
  field enabled bool "Whether the world takes its ticks in lockstep when in a room."
  field players u32 "The seats: the peers lockstep waits for before it starts, and whose inputs each tick waits for." {
    default 2
    min 1
    max 16
  }
  field localPlayer u32 "This peer's seat, the og.PlayerInput.player its input drives; `players` while it watches (a peer beyond the seats). 0 before the start." {
    readonly true
  }
  field inputDelay u32 "Ticks between a player's input and the tick it acts on: the time the input has to cross the room. 2 (33 ms at 60 Hz) on a LAN, 4 to 6 over the internet." {
    default 2
    min 1
    max 30
    unit ticks
  }
  field hashEvery u32 "How often the peers exchange their world hashes, in ticks (every tick is hashed, so the first divergent tick is exact); 0 compares nothing." {
    default 10
    min 0
    max 600
    unit ticks
  }
  field running bool "Whether lockstep runs: the room filled the seats and the authority started (until this peer leaves the room or turns it off)." {
    readonly true
  }
  field mode NetMode "lockstep (the default: every tick waits for every input) or rollback (ticks run on predicted inputs and are rewound when an input differs; set before the room starts, the authority's is the room's)."
  field stallTimeout u32 "Milliseconds a running peer may stay silent before its seat is dropped (og.SeatDropped); 0 never drops a peer still in the room, so the room waits for it as long as it takes. The authority's is the room's." {
    default 5000
    min 0
    max 600000
    unit ms
  }
  field rejoin bool "Whether a dropped player that comes back (or a newcomer of the same name) takes its seat again (og.SeatRejoined); when off, a seat once dropped stays empty and newcomers take only seats never held. The authority's is the room's." {
    default true
  }
  field maxPrediction u32 "In rollback, the most ticks this peer runs past the last tick whose inputs are all in before it waits (a peer also predicts no further than it can re-simulate in half a frame). 8 (133 ms at 60 Hz) covers most internet latency on top of inputDelay." {
    default 8
    min 1
    max 30
    unit ticks
  }
}
```

<sub>`api.ogt` line 29320</sub>

<a id="record-PlayerInput"></a>

### `og.PlayerInput` (component)

One player's input for this tick, the same on every peer: the engine writes every field but `player` at the start of each tick from the seat's input (zero for a seat nobody holds, and before its holder's first input). Its packed bytes are what crosses the room, read through this layout.

Token `og.PlayerInput`, registered under the schema name `og.PlayerInput`; its value type is `PlayerInput`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `player` | `number` | `0` | — | — | — | The seat whose input this entity carries (0 to og.Lockstep.players - 1); the game sets it when it spawns the player. |
| `buttons` | `number` | `0` | — | — | — | The buttons held, one bit each, as the game numbers them (bit 0 fire, bit 1 dash, ...). |
| `x` | `number` | `0` | -1 … 1 | — | — | The move axis, left -1 to right 1. |
| `y` | `number` | `0` | -1 … 1 | — | — | The move axis, back -1 to forward 1. |
| `aimX` | `number` | `0` | — | — | — | A second axis (aim, a cursor's world x), as the game uses it. |
| `aimY` | `number` | `0` | — | — | — | A second axis (aim, a cursor's world y), as the game uses it. |
| `tick` | `number` | `0` | — | — | read only | The tick these values are for: the tick now, once the engine wrote them. |

```ogt
component PlayerInput {
  doc "One player's input for this tick, the same on every peer: the engine writes every field but `player` at the start of each tick from the seat's input (zero for a seat nobody holds, and before its holder's first input). Its packed bytes are what crosses the room, read through this layout."
  field player u32 "The seat whose input this entity carries (0 to og.Lockstep.players - 1); the game sets it when it spawns the player."
  field buttons u32 "The buttons held, one bit each, as the game numbers them (bit 0 fire, bit 1 dash, ...)."
  field x f32 "The move axis, left -1 to right 1." {
    min -1
    max 1
  }
  field y f32 "The move axis, back -1 to forward 1." {
    min -1
    max 1
  }
  field aimX f32 "A second axis (aim, a cursor's world x), as the game uses it."
  field aimY f32 "A second axis (aim, a cursor's world y), as the game uses it."
  field tick u32 "The tick these values are for: the tick now, once the engine wrote them." {
    readonly true
  }
}
```

<sub>`api.ogt` line 29364</sub>

<a id="record-Desync"></a>

### `og.Desync` (event)

This peer's world and another's differ: sent once, at the start of the tick after the hashes showed it. The probe's `desync` dump (`dump_desync`) names the component and field that differ, once the other peer's snapshot of that tick arrived.

Token `og.Desync`, registered under the schema name `og.Desync`; its value type is `Desync`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `tick` | `number` | `0` | — | — | — | The first divergent tick: the world after it differs (og.Time.tick of the snapshots compared). |
| `peer` | `number` | `0` | — | — | — | The first peer whose world differs from this one. |
| `peers` | `number` | `0` | — | — | — | How many peers' worlds differ from this one. |

```ogt
event Desync {
  doc "This peer's world and another's differ: sent once, at the start of the tick after the hashes showed it. The probe's `desync` dump (`dump_desync`) names the component and field that differ, once the other peer's snapshot of that tick arrived."
  field tick u32 "The first divergent tick: the world after it differs (og.Time.tick of the snapshots compared)."
  field peer u32 "The first peer whose world differs from this one."
  field peers u32 "How many peers' worlds differ from this one."
}
```

<sub>`api.ogt` line 29383</sub>

<a id="record-SeatDropped"></a>

### `og.SeatDropped` (event)

A seat's player was dropped: it left the room, or stalled past og.Lockstep.stallTimeout. Sent at the start of the first tick the seat is empty, the same tick on every peer (a hashed event: the confirmed world holds it); from then on its og.PlayerInput is the zero value until og.SeatRejoined.

Token `og.SeatDropped`, registered under the schema name `og.SeatDropped`; its value type is `SeatDropped`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `seat` | `number` | `0` | — | — | — | The seat, og.PlayerInput.player. |
| `peer` | `number` | `0` | — | — | — | The peer that held it. |
| `tick` | `number` | `0` | — | — | — | The first tick the seat is empty (og.Time.tick now). |
| `left` | `boolean` | `false` | — | — | — | Whether the peer left the room (else it stalled and may come back). |

```ogt
event SeatDropped {
  doc "A seat's player was dropped: it left the room, or stalled past og.Lockstep.stallTimeout. Sent at the start of the first tick the seat is empty, the same tick on every peer (a hashed event: the confirmed world holds it); from then on its og.PlayerInput is the zero value until og.SeatRejoined."
  field seat u32 "The seat, og.PlayerInput.player."
  field peer u32 "The peer that held it."
  field tick u32 "The first tick the seat is empty (og.Time.tick now)."
  field left bool "Whether the peer left the room (else it stalled and may come back)."
}
```

<sub>`api.ogt` line 29390</sub>

<a id="record-SeatRejoined"></a>

### `og.SeatRejoined` (event)

A seat held before is taken again, by its returning player (og.Lockstep.rejoin) or a newcomer: sent at the start of the first tick of its new holder's input, the same tick on every peer (hashed).

Token `og.SeatRejoined`, registered under the schema name `og.SeatRejoined`; its value type is `SeatRejoined`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `seat` | `number` | `0` | — | — | — | The seat. |
| `peer` | `number` | `0` | — | — | — | The peer that holds it now. |
| `tick` | `number` | `0` | — | — | — | The first tick of its input (og.Time.tick now). |

```ogt
event SeatRejoined {
  doc "A seat held before is taken again, by its returning player (og.Lockstep.rejoin) or a newcomer: sent at the start of the first tick of its new holder's input, the same tick on every peer (hashed)."
  field seat u32 "The seat."
  field peer u32 "The peer that holds it now."
  field tick u32 "The first tick of its input (og.Time.tick now)."
}
```

<sub>`api.ogt` line 29398</sub>

<a id="record-PeerStalled"></a>

### `og.PeerStalled` (event)

A running peer has sent nothing for a second (or half of stallTimeout, if shorter): for a "waiting for player" message. This peer's own (wall time decides it): never hashed, and a peer that speaks again is reported again the next time it falls silent.

Token `og.PeerStalled`, registered under the schema name `og.PeerStalled`; its value type is `PeerStalled`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `peer` | `number` | `0` | — | — | — | The silent peer. |
| `seat` | `number` | `0` | — | — | — | Its seat; og.Lockstep.players for a spectator. |
| `silentMs` | `number` | `0` | — | `ms` | — | How long it has been silent, in milliseconds. |

```ogt
event PeerStalled {
  doc "A running peer has sent nothing for a second (or half of stallTimeout, if shorter): for a \"waiting for player\" message. This peer's own (wall time decides it): never hashed, and a peer that speaks again is reported again the next time it falls silent."
  field peer u32 "The silent peer."
  field seat u32 "Its seat; og.Lockstep.players for a spectator."
  field silentMs u32 "How long it has been silent, in milliseconds." {
    unit ms
  }
}
```

<sub>`api.ogt` line 29405</sub>

## Types

<a id="type-NetMode"></a>

### `NetMode` (enum)

How a room's peers take their ticks (og.Lockstep.mode).

A string: one of the variants below. The zero value is `"lockstep"`.

| Variant | Doc |
|---|---|
| `"lockstep"` | Each tick waits until every seated player's input for it is in: no prediction, no rollback; a late input holds the world. |
| `"rollback"` | A tick runs on a missing input predicted as that player's last one, up to maxPrediction ticks past the last tick whose inputs are all in; when the real input differs the world is rewound to its own snapshot and the ticks run again within the frame (GGPO-style). Every peer agrees on the confirmed world, so hashes and og.Desync cover it alone. |

```ogt
enum NetMode {
  doc "How a room's peers take their ticks (og.Lockstep.mode)."
  variant lockstep "Each tick waits until every seated player's input for it is in: no prediction, no rollback; a late input holds the world."
  variant rollback "A tick runs on a missing input predicted as that player's last one, up to maxPrediction ticks past the last tick whose inputs are all in; when the real input differs the world is rewound to its own snapshot and the ticks run again within the frame (GGPO-style). Every peer agrees on the confirmed world, so hashes and og.Desync cover it alone."
}
```

<sub>`api.ogt` line 29307</sub>

<a id="type-InputDesc"></a>

### `InputDesc` (desc struct)

This peer's player's input: og.PlayerInput's fields but `player` and `tick`.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `buttons` | `number` | `0` | — | — | — | The buttons held, one bit each. |
| `x` | `number` | `0` | — | — | — | The move axis, -1 to 1. |
| `y` | `number` | `0` | — | — | — | The move axis, -1 to 1. |
| `aimX` | `number` | `0` | — | — | — | A second axis. |
| `aimY` | `number` | `0` | — | — | — | A second axis. |

```ogt
struct InputDesc {
  doc "This peer's player's input: og.PlayerInput's fields but `player` and `tick`."
  desc true
  field buttons u32 "The buttons held, one bit each."
  field x f32 "The move axis, -1 to 1."
  field y f32 "The move axis, -1 to 1."
  field aimX f32 "A second axis."
  field aimY f32 "A second axis."
}
```

<sub>`api.ogt` line 29414</sub>

<a id="type-NetShapeDesc"></a>

### `NetShapeDesc` (desc struct)

A game's own input shape (og.net.shape): the record each seat's input fills in place of og.PlayerInput, and the commands a seat gives on single ticks (an RTS's orders).

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `input` | `AnyComponent \| null` | `null` | — | — | — | The component every seat's input fills, one entity per player, in place of og.PlayerInput: a game's own with a u32 `player` field (the seat it carries, set by the game) and any others (a u32 `tick`, when it has one, is written as og.PlayerInput's is). og.net.submitInput gives this peer's. Null keeps og.PlayerInput. |
| `command` | `AnyComponent \| null` | `null` | — | — | — | The component whose values are the commands (og.net.command, read with og.net.commands): a game's own with a u32 `player` field, which the engine sets to the seat that gave the command. Used as a record shape only: no entity needs to carry it. Null: no commands. |
| `maxCommands` | `number` | `32` | 1 … 256 | — | — | The most commands one seat gives for one tick; og.net.command refuses one more. |

```ogt
struct NetShapeDesc {
  doc "A game's own input shape (og.net.shape): the record each seat's input fills in place of og.PlayerInput, and the commands a seat gives on single ticks (an RTS's orders)."
  desc true
  field input Option.AnyComponent "The component every seat's input fills, one entity per player, in place of og.PlayerInput: a game's own with a u32 `player` field (the seat it carries, set by the game) and any others (a u32 `tick`, when it has one, is written as og.PlayerInput's is). og.net.submitInput gives this peer's. Null keeps og.PlayerInput."
  field command Option.AnyComponent "The component whose values are the commands (og.net.command, read with og.net.commands): a game's own with a u32 `player` field, which the engine sets to the seat that gave the command. Used as a record shape only: no entity needs to carry it. Null: no commands."
  field maxCommands u32 "The most commands one seat gives for one tick; og.net.command refuses one more." {
    default 32
    min 1
    max 256
  }
}
```

<sub>`api.ogt` line 29424</sub>

<a id="type-LockstepSeat"></a>

### `LockstepSeat` (struct)

One peer's hold on a seat.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `seat` | `number` | `0` | — | — | — | The seat. |
| `peer` | `number` | `0` | — | — | — | The peer that holds (or held) it. |
| `since` | `number` | `0` | — | `ticks` | — | The first tick of its input. |
| `until` | `number \| null` | `null` | — | `ticks` | — | The last tick of its input, once it was dropped. |
| `left` | `boolean` | `false` | — | — | — | Whether it was dropped because it left the room (else it stalled). |

```ogt
struct LockstepSeat {
  doc "One peer's hold on a seat."
  field seat u32 "The seat."
  field peer u32 "The peer that holds (or held) it."
  field since f64 "The first tick of its input." {
    unit ticks
  }
  field until Option.f64 "The last tick of its input, once it was dropped." {
    unit ticks
  }
  field left bool "Whether it was dropped because it left the room (else it stalled)."
}
```

<sub>`api.ogt` line 29436</sub>

<a id="type-LockstepStatus"></a>

### `LockstepStatus` (struct)

Lockstep on this peer, as the probe and a HUD read it.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `running` | `boolean` | `false` | — | — | — | Whether lockstep runs. |
| `peer` | `number \| null` | `null` | — | — | — | This peer's number; null in no room. |
| `peers` | `readonly number[]` | `[]` | — | — | — | The peers present, by number. |
| `startTick` | `number \| null` | `null` | — | `ticks` | — | The tick lockstep started (or this peer joined it) at. |
| `players` | `number \| null` | `null` | — | — | — | The seats, while running. |
| `seats` | `readonly LockstepSeat[]` | `[]` | — | — | — | Every seat's holds, in order. |
| `spectators` | `readonly number[]` | `[]` | — | — | — | The running peers without a seat. |
| `waitingFor` | `readonly number[]` | `[]` | — | — | — | The seats whose input for the next tick has not arrived. |
| `waits` | `number` | `0` | — | — | — | Tick boundaries this peer waited at for an input. |
| `desync` | `number \| null` | `null` | — | — | — | The first divergent tick, once og.Desync was sent. |
| `rollback` | `boolean` | `false` | — | — | — | Whether the room runs in rollback (og.Lockstep.mode, the authority's). |
| `maxPrediction` | `number \| null` | `null` | — | `ticks` | — | The room's maxPrediction while running in rollback; 0 in lockstep. |
| `confirmedTick` | `number \| null` | `null` | — | `ticks` | — | The last tick whose every seat's input has arrived. |
| `finalTick` | `number \| null` | `null` | — | `ticks` | — | The last tick whose world is final: run on real inputs only, the world og.net.finalHash and the hashes the peers compare cover. |
| `acknowledged` | `number \| null` | `null` | — | `ticks` | — | The last tick through which every other running peer has all of this peer's inputs (each input message acknowledges); null alone or before the start. |
| `predicted` | `number` | `0` | — | `ticks` | — | Ticks the world now runs past its last confirmed tick (0 in lockstep). |
| `rollbacks` | `number` | `0` | — | — | — | Rewinds this peer made since lockstep started. |
| `resimulatedTicks` | `number` | `0` | — | `ticks` | — | Ticks this peer ran again after a rewind. |
| `stalled` | `readonly number[]` | `[]` | — | — | — | The running peers silent for a second or more (og.PeerStalled). |
| `drops` | `number` | `0` | — | — | — | Seats (and spectators) this peer has seen dropped since it joined the room. |
| `snapshotBytes` | `number` | `0` | — | `bytes` | — | The bytes of the snapshots kept for a rewind: one per tick not yet final. |

```ogt
struct LockstepStatus {
  doc "Lockstep on this peer, as the probe and a HUD read it."
  field running bool "Whether lockstep runs."
  field peer Option.u32 "This peer's number; null in no room."
  field peers List.u32 "The peers present, by number."
  field startTick Option.f64 "The tick lockstep started (or this peer joined it) at." {
    unit ticks
  }
  field players Option.u32 "The seats, while running."
  field seats List.LockstepSeat "Every seat's holds, in order."
  field spectators List.u32 "The running peers without a seat."
  field waitingFor List.u32 "The seats whose input for the next tick has not arrived."
  field waits f64 "Tick boundaries this peer waited at for an input."
  field desync Option.u32 "The first divergent tick, once og.Desync was sent."
  field rollback bool "Whether the room runs in rollback (og.Lockstep.mode, the authority's)."
  field maxPrediction Option.u32 "The room's maxPrediction while running in rollback; 0 in lockstep." {
    unit ticks
  }
  field confirmedTick Option.f64 "The last tick whose every seat's input has arrived." {
    unit ticks
  }
  field finalTick Option.f64 "The last tick whose world is final: run on real inputs only, the world og.net.finalHash and the hashes the peers compare cover." {
    unit ticks
  }
  field acknowledged Option.f64 "The last tick through which every other running peer has all of this peer's inputs (each input message acknowledges); null alone or before the start." {
    unit ticks
  }
  field predicted u32 "Ticks the world now runs past its last confirmed tick (0 in lockstep)." {
    unit ticks
  }
  field rollbacks f64 "Rewinds this peer made since lockstep started."
  field resimulatedTicks f64 "Ticks this peer ran again after a rewind." {
    unit ticks
  }
  field stalled List.u32 "The running peers silent for a second or more (og.PeerStalled)."
  field drops f64 "Seats (and spectators) this peer has seen dropped since it joined the room."
  field snapshotBytes f64 "The bytes of the snapshots kept for a rewind: one per tick not yet final." {
    unit bytes
  }
}
```

<sub>`api.ogt` line 29449</sub>

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

<sub>`api.ogt` line 29313</sub>
