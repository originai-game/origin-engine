<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `net` module

Deterministic lockstep over a room (docs/design.md §2.10): every peer runs the whole world, and what crosses the room is each player's input, one per tick, and the world's hash. Turn it on with og.Lockstep (`enabled`, `players`); join a room (og.platform.roomJoin, or `og run --room ws://127.0.0.1:P/<room>` with `og relay` natively). Until the room holds `players` peers the world runs freely on each (a lobby, the local input driving the local player); then the authority (the lowest peer number) starts: every other peer takes its world's snapshot, the seats go to the peers in number order, and from then on tick t runs only when every seated player's input for t is in. A peer that joins a running room takes a free seat (or watches) from a snapshot a running peer hands it.

Inputs are og.PlayerInput: one entity per player carries it with `player` set to its seat; the engine writes every other field at the start of each tick from that seat's input for the tick. A system reads the inputs from og.PlayerInput, never from og.Input, and gives this peer's own with og.net.submit (from a system that reads og.Input: what the local player holds now, sent for the tick inputDelay ticks on). Without a room, or with lockstep off, og.net.submit drives the local player's og.PlayerInput the next tick, so the same game plays alone.

Every tick each peer hashes its world without its own session (og.Input, og.Pointers, og.Gamepad, og.Audio, og.Camera, og.Lockstep, the input, room, platform, asset and stream events, the UI's focus and hover) and every hashEvery ticks sends the hashes since its last batch; the first tick whose hashes differ is reported once as og.Desync, and the probe's `desync` dump (`dump_desync`) names it with the component-level diff of the two peers' snapshots of that tick: the answer to what is not deterministic (Math.random, Date.now, a wall clock, iteration over a Set of objects, a module variable, input read from og.Input).

- **TypeScript**: functions are the namespace `og.net`; components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Skill**: `og-game`, which teaches the module (how much of it to use, and how to prove it).
- **Index**: [every module and name](index.md).

```ogt
module net {
  # declarations follow, each quoted below
```

<sub>`api.ogt` line 13776</sub>

## Contents

| Name | Kind | Summary |
|---|---|---|
| [`og.Lockstep`](#record-Lockstep) | resource | Lockstep's settings and this peer's place. The game sets enabled, players, inputDelay and hashEvery (the authority's are the room's once it starts: changes while running wait for the next start); the engine sets localPlayer and running. It is this peer's own: never hashed, never compared. |
| [`og.PlayerInput`](#record-PlayerInput) | component | One player's input for this tick, the same on every peer: the engine writes every field but `player` at the start of each tick from the seat's input (zero for a seat nobody holds, and before its holder's first input). Its packed bytes are what crosses the room, read through this layout. |
| [`og.Desync`](#record-Desync) | event | This peer's world and another's differ: sent once, at the start of the tick after the hashes showed it. The probe's `desync` dump (`dump_desync`) names the component and field that differ, once the other peer's snapshot of that tick arrived. |
| [`InputDesc`](#type-InputDesc) | desc struct | This peer's player's input: og.PlayerInput's fields but `player` and `tick`. |
| [`LockstepSeat`](#type-LockstepSeat) | struct | One peer's hold on a seat. |
| [`LockstepStatus`](#type-LockstepStatus) | struct | Lockstep on this peer, as the probe and a HUD read it. |
| [`og.net.submit`](#fn-submit) | function | Give this peer's player's input: in lockstep, for the tick inputDelay ticks from the one running (sent to every peer when the tick ends); otherwise, for the next tick. It holds until the next call, so a game calls it once a tick from a system that reads og.Input (or og.Gamepad, or the UI's touch controls). Callable from a system. |
| [`og.net.status`](#fn-status) | function | Lockstep on this peer: running, the seats and who it waits for. |
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

<sub>`api.ogt` line 13874</sub>

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

<sub>`api.ogt` line 13881</sub>

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

<sub>`api.ogt` line 13887</sub>

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
}
```

<sub>`api.ogt` line 13782</sub>

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

<sub>`api.ogt` line 13810</sub>

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

<sub>`api.ogt` line 13829</sub>

## Types

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

<sub>`api.ogt` line 13836</sub>

<a id="type-LockstepSeat"></a>

### `LockstepSeat` (struct)

One peer's hold on a seat.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `seat` | `number` | `0` | — | — | — | The seat. |
| `peer` | `number` | `0` | — | — | — | The peer that holds (or held) it. |
| `since` | `number` | `0` | — | `ticks` | — | The first tick of its input. |
| `until` | `number \| null` | `null` | — | `ticks` | — | The last tick of its input, once it left. |

```ogt
struct LockstepSeat {
  doc "One peer's hold on a seat."
  field seat u32 "The seat."
  field peer u32 "The peer that holds (or held) it."
  field since f64 "The first tick of its input." {
    unit ticks
  }
  field until Option.f64 "The last tick of its input, once it left." {
    unit ticks
  }
}
```

<sub>`api.ogt` line 13846</sub>

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
}
```

<sub>`api.ogt` line 13858</sub>
