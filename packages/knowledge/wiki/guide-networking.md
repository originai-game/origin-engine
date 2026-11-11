<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `networking` guide: Networking

How to choose and run a net mode: deterministic lockstep or rollback over a room, the input delay and prediction that hide latency, seats that drop and rejoin, desync detection and what to do about one, and rooms through og.platform. Every peer runs the whole world; what crosses the room is each player's input, one per tick, and the world's hash. The rules a simulation must keep to stay in step are in `determinism §1` and `determinism §3`.

The API: `og api show net`; rooms are og.platform's (`og api show platform`).

<a id="guide-networking-1"></a>
### `networking §1` Choosing a net mode

| | lockstep (the default) | rollback |
|---|---|---|
| a late input | holds the world until it arrives | the tick runs on a prediction (the player's last input) and is re-run when the real input differs |
| feel at 100 ms | every input lags by `inputDelay` | local input answers within `inputDelay`, others snap when corrected |
| cost | one world per tick | a snapshot per unconfirmed tick and up to `maxPrediction` ticks re-run in a frame |
| choose for | RTS, turn-based, co-op, many units; at most 16 seats (engine limit) | fighting, sports, action with few players |

- Both modes keep the same seats, inputs and hashes. Set og.Lockstep's
  `mode` before the room starts; the authority's is the room's.
- Rollback re-runs systems: a world whose tick is expensive (thousands of
  bodies) cannot re-simulate eight ticks in a frame. A peer predicts no
  further than it can re-simulate in half a frame.
- The peers agree on the confirmed world only (og.net.finalHash); a
  predicted world is this peer's guess.

<a id="guide-networking-2"></a>
### `networking §2` Starting a room

```ts
// GameDesc.start
const ls = world.resource(og.Lockstep).value!;
ls.enabled = true; ls.players = 2; ls.inputDelay = 3;
og.platform.roomJoin(world, roomId);   // or og.platform.roomCreate, og.platform.quickPlay
```

- og.RoomJoined answers the join; og.platform's rooms carry the peers, and
  lockstep rides on them.

- Natively, `og relay` plus `og run --room ws://127.0.0.1:P/<room>` plays
  a room by hand.
- Until the room holds `players` peers the world runs freely on each: a
  lobby, the local input driving the local player. Then the authority
  (the lowest peer number) starts: every other peer takes its world's
  snapshot, and the seats go to the peers in number order.
- Once the room is full the other peers' worlds hold for that snapshot,
  for at most `stallTimeout` (5 s when it is 0), so a slow authority's
  start does not find them far into a lobby it replaces.
- A peer that joins a running room takes a free seat, or watches, from a
  snapshot a running peer hands it.
- The authority's og.Lockstep settings are the room's once it starts;
  changes while running wait for the next start. og.Lockstep is this
  peer's own: never hashed, never compared.

<a id="guide-networking-3"></a>
### `networking §3` Inputs and input delay

Exactly one system reads og.Input (or og.Gamepad, or the UI's touch
controls) and calls og.net.submit once a tick; every other system reads
og.PlayerInput. Alone, or with lockstep off, the submitted input drives the
local player's og.PlayerInput the next tick, so the same game plays alone.

```ts
const Send = defineSystem("Send", { reads: [og.Input], writes: [] }, (q, time, world) => {
  const move = og.input.actionVector(world, "move");
  og.net.submit(world, { x: move.x, y: move.y, buttons: og.input.isActionPressed(world, "fire") ? 1 : 0 });
});
```

- One entity per player carries og.PlayerInput with `player` set to its
  seat; the engine writes every other field at the start of each tick from
  that seat's input for the tick. Its packed bytes are what crosses the
  room.
- A game with richer input declares its own with og.net.shape, in setup
  or start, before the room starts and the same on every peer (a peer
  whose shape differs is refused the room's start). Orders that act on
  one tick only (an RTS's orders, a card played) are commands
  (og.net.command, read with og.net.commands): never repeated by a
  prediction.
- An input acts `inputDelay` ticks after it is submitted: the time it has
  to cross the room. Local feedback (a sound, a flash, an animation start)
  may react at once from the presentation; the simulation may not.

| og.Lockstep | Value | Source |
|---|---|---|
| `inputDelay` | 2 ticks (33 ms at 60 Hz) | engine default |
| `inputDelay`, recommended | 2 on a LAN, 4 to 6 over the internet | engine guidance (IDL at 9ba5b0957) |
| `maxPrediction` | 8 ticks (133 ms at 60 Hz) | engine default |
| `maxPrediction`, recommended | 8 covers most internet latency on top of `inputDelay` | engine guidance (IDL at 9ba5b0957) |
| `players` | at most 16 | engine limit |

  An input delay must cover the room's round trip, or lockstep waits and
  rollback corrects more; a lower delay feels better and rolls back more.

<a id="guide-networking-4"></a>
### `networking §4` Rollback

- When a real input differs from the prediction, the world is rewound to
  the engine's own snapshot of that tick and runs forward again within the
  frame. The engine plays each sound once and snaps the view.
- Systems run again for re-simulated ticks. A game's own presentation
  triggered from a system (a screen shake, a haptic, a particle burst)
  checks og.net.resimulating first, or comes from events the engine
  deduplicates.
- A command given while a tick is re-simulated is ignored: its first run
  gave it already.
- Inputs cross the room unreliably and are repeated until acknowledged,
  so a lost message costs time, not the room.
- og.net.status shows the cost: `predicted`, `rollbacks`,
  `resimulatedTicks` and `snapshotBytes`. Rollbacks on most ticks mean the
  delay is too low for the connection.

<a id="guide-networking-5"></a>
### `networking §5` Stalls, drops and rejoin

- A peer that goes silent (a frozen tab, a lost connection the room has
  not noticed yet) is reported as og.PeerStalled after a second (or half of
  `stallTimeout`, if shorter): show "waiting for player" then. It is this
  peer's own report, never hashed, and repeats each time the peer falls
  silent again.
- After `stallTimeout` the seat is dropped: the authority (the lowest
  running peer that is not itself silent) agrees with every other running
  peer how far the seat's inputs reach, and every peer empties the seat
  from the tick after its last input. og.SeatDropped arrives at one tick
  everywhere and the room plays on. A peer that leaves the room is dropped
  the same way at once.
- A dropped seat's input is og.PlayerInput's zero value, so a held move or
  fire stops. A game that wants a bot there reads og.SeatDropped and drives
  that seat itself, the same way on every peer; or it ends the match.
- With og.Lockstep's `rejoin`, a dropped player that comes back (the same
  peer, or a newcomer of the same name) takes its seat again from the
  authority's world, with og.SeatRejoined at one tick on every peer.
- Wall time decides only when a peer is reported and dropped, never what
  the confirmed world is. `stallTimeout` 0 never drops a peer still in the
  room: the room waits as long as it takes.

<a id="guide-networking-6"></a>
### `networking §6` Desync detection and recovery

Every tick each peer hashes its world without its own session: og.Input,
og.Pointers, og.Gamepad, og.PointerLock, og.Audio, og.Camera, og.Lockstep,
the gameplay actions of og.input, the input, room, platform, asset and
stream events, and the UI's focus and hover. Every `hashEvery` ticks it
sends the hashes since its last batch; the first tick whose hashes differ
is reported once as og.Desync.

- og.net.hash is that shared hash; the world dump's `hash` covers
  everything and differs between peers.
- The probe's `desync` dump (`dump_desync`) names the first divergent tick
  with the component-level diff of the two peers' snapshots of it, once the
  other peer's snapshot arrived. Its first entry is the first difference in
  the local world's storage order. Read it before guessing.
- The usual causes: Math.random, Date.now or another wall clock, iteration
  over a Set of objects, a module variable, input read from og.Input or
  og.input's actions outside the one submitting system, and Math's
  transcendental functions (`determinism §1`).
- `hashEvery` 10 finds the tick exactly (every tick is hashed) at little
  traffic; 0 compares nothing and hides desyncs.
- Recovery: a desync is a bug to fix, not to paper over. A shipped game that
  meets one ends the match rather than keep playing two worlds.

<a id="guide-networking-7"></a>
### `networking §7` Testing a room

- `PlaytestDesc.room` runs several processes in one room with distinct
  scripts per seat; assert on `run.lockstep` and the peers' `netHash`.
- Two peers fed the same inputs agreeing proves little; feed them
  different ones, and drop one mid-match to test og.SeatDropped.
- `runs` is refused with `room`; each `then` step relaunches every peer of
  the room at once.
