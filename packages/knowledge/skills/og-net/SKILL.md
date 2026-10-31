---
name: og-net
description: Multiplayer on Origin Engine by deterministic lockstep over a room — og.Lockstep and og.PlayerInput, one system submitting inputs with og.net.submit, the rules that keep every peer's world identical, og relay and --room for native runs, Play links that join a room, playtests that play themselves in a room (netHash, run.peers), and finding a desync with og.Desync and dump_desync. Use when two or more players share a game over a room, or when peers' worlds part.
---

Two players on Origin Engine run the same world on every peer and send
only inputs over a room; og-platform-sdk §2 lists the room calls.

- §1 Two players over a room: the three rules, running and testing in a
  room, and a desync.

## 1. Two players over a room: lockstep

To play together, every peer runs the whole world and only inputs cross
the room (`og api show net`; the rooms themselves are og.platform's, taught by
the og-platform-sdk skill). Follow three rules:

1. In `start`, set `og.Lockstep`: `enabled = true`, `players`,
   `inputDelay` (2 on a LAN, 4 to 6 over the internet) and `hashEvery`
   (10). Spawn one entity per player with `og.PlayerInput { player: seat }`.
2. Read `og.Input`, `og.Gamepad` and the UI's touch controls in **one**
   system, and hand them over with
   `og.net.submit(world, { x, y, buttons, aimX, aimY })` every tick.
3. Every system that changes the world reads `og.PlayerInput` (its `x`,
   `y` and `buttons` bits), the world and `og.Rng`, and nothing else: no
   `Math.random()`, no `Date.now()`, no module variables, no
   `og.RoomMessage`. Draw per-peer things (whose turn it is, "waiting")
   with og.ui only, and keep them out of the world.

```ts
const Submit = defineSystem("Submit", { reads: [og.Input] }, (_q, _t, world) => {
  const input = must(world.resource(og.Input));
  must(og.net.submit(world, { x: input.axis("KeyA", "KeyD"), y: input.axis("KeyS", "KeyW"), buttons: input.down("Space") ? 1 : 0 }));
});
const Move = defineSystem("Move", { reads: [og.PlayerInput], writes: [og.Transform] }, (q, time) => {
  for (const a of q.archetypes()) {
    const pin = a.column(og.PlayerInput);
    const p = a.column(og.Transform).position;
    for (let i = 0; i < a.count; i++) p[i * 3] += pin.x[i]! * 6 * time.dt;
  }
});
```

Alone, or with lockstep off, `submit` drives the local player on the next
tick, so the same code plays single-player. Natively, run `og relay` and
then `og run . --room ws://127.0.0.1:47900/<room>` in two terminals; on
the web the shell carries the platform's rooms, and a Play link with
`?room=<id>` puts the page in that room at load (the game hears
og.RoomJoined as for its own roomJoin; `og.platform.roomInfo(world).link`
names the link's room), so a game that plays over a room need not ask for
one when the link named it. A playtest plays itself in
a room with `room: { players: 2, commands: [...] }`: its runs are `run`
and `run.peers`, and each has `netHash` and `lockstep`. Equal `netHash`
values at the same tick mean the peers stayed in step. Once lockstep runs
a script's steps are on the room's shared ticks (a peer that took the
authority's snapshot keeps its steps on theirs), so end each script with
`quit` (or give `room.ticks`) and every place stops on one tick; a `then`
step relaunches every place at once into a room of its own, its runs in
`run.then[n]` and `run.then[n].peers`.

When the worlds part, `og.Desync { tick, peer }` fires once and
`og.net.status(world).desync` holds the tick. Then
`--commands "...:dump_desync:d.json"` names the first divergent tick and
the entity, component and field that differ, with both values: fix the
system that writes that field. The archetype `templates/archetypes/arena`
(Coin Duel) is the worked example.
