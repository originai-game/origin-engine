<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `shared-worlds` guide: Shared worlds

How to let the visitors of one world see each other: what a shared world shares and what it does not, joining, drawing the others, moving your own avatar within the server's rules, running a world server for development and self-hosting, and replaying a session. For games where every peer runs the same simulation, use lockstep instead (`networking §1`).

The API: `og api show shared`.

<a id="guide-shared-worlds-1"></a>
### `shared-worlds §1` Shared world or lockstep

| | lockstep room (`networking`) | shared world |
|---|---|---|
| who simulates | every peer, the whole world, in step | each client its own world; a server holds the avatars |
| what crosses | every player's input, every tick | each avatar's state, to the clients near it |
| players | at most 16 | 200 in one area, 2000 per server, more as more servers |
| a slow player | holds or rolls back everyone | is drawn late; nobody waits |
| cheating | a peer can only lie about its input | the server checks each move against its kind's speed |

- A shared world shares presence: where each visitor is, where it faces,
  how fast it moves, and a few bytes of the game's own (a locomotion
  mode, an appearance). Doors, pickups and enemies stay each client's own.
- The two do not mix: a world in lockstep cannot join a shared world.

<a id="guide-shared-worlds-2"></a>
### `shared-worlds §2` Joining

```ts
// GameDesc.setup: the avatar's extra fields, before joining.
og.shared.shape(world, { avatar: Look });
// The player's entity: its og.Transform is what is shared.
const me = world.spawn({ name: "me" }).value!;
world.insert(me, og.Transform, { position: spawnAt });
world.insert(me, og.SharedAvatar, { kind: 0 });
world.insert(me, Look, { mode: 0, model: 3 });
og.shared.join(world, { world: "main" });
```

- Move the avatar as always; the engine proposes its state at
  og.SharedWorld.sendRateHz and needs nothing else. Yaw is read from the
  rotation about y, velocity from the moves.
- The avatar component is packed as it is: keep it small (enums and ids,
  not floats you could derive). It is 32 bytes at most.
- Where the connection comes from is the host's: natively
  `og run --shared ws://127.0.0.1:47910 --shared-token-file <f>`; on a page,
  its session provider. The game never names a URL or a token.
- og.SharedJoined says the server took the client in;
  og.SharedLeft says why a session ended. A lost connection reconnects by
  itself, and the server keeps the avatar 10 s for it to come back.

<a id="guide-shared-worlds-3"></a>
### `shared-worlds §3` Drawing the others

Every visitor in sight is an og.RemoteAvatar entity with an og.Transform
and, after og.shared.shape, the game's avatar component.

- Give it visuals when it enters (og.PresenceEntered), and leave the rest
  to the engine: the entity is despawned when it leaves (og.PresenceLeft).
- It is drawn `interpDelayMs` behind the newest state (100 ms by default):
  smooth over one lost update. Lower it for faster reactions at the cost
  of visible stalls on a poor connection; 50 ms is the least that hides one
  lost update at 20 Hz.
- A teleported avatar (og.RemoteAvatar.jumped) appears at once; play an
  effect there rather than moving it across.
- The server sends the 64 nearest within two 128 m cells, nearer ones more
  often. Draw names and full detail for the nearest 16 or so; a crowd
  past that reads well as simplified figures.

<a id="guide-shared-worlds-4"></a>
### `shared-worlds §4` Your own avatar and the server's rules

- The server takes a move when its distance fits the kind's top speed
  (with half a second of slack for bunched updates) and the place is
  inside the world's bounds. A refused move comes back as
  og.SharedCorrected, with the avatar already put back where the server
  holds it plus what it moved since.
- Corrections mean a rule and the game disagree: give each movement mode
  its own kind and the server a top speed per kind a little above the
  game's (`maxSpeed` in the world server's config).
- Jumps across the map (a map click, a portal) go through og.shared.teleport,
  never a plain transform write, or the server refuses them as too fast.

<a id="guide-shared-worlds-5"></a>
### `shared-worlds §5` A world server for development and self-hosting

```sh
og world-server --auth none --dev                        # local only: players name themselves
og world-server token --secret-file s --audience my-game --sub p1 > t1
og world-server --auth hs256 --secret-file s --audience my-game
og run --shared ws://127.0.0.1:47910 --shared-token-file t1
```

- `--auth hs256`: the game's own token endpoint signs short-lived tokens
  with the same secret (a page's `tokenEndpoint` provider asks it).
  `--auth jwks`: an issuer that publishes its keys. Keep tokens to five
  minutes; a reconnect asks for a new one.
- The server's world config (an `.ogt` file, `--config`) sets the cell
  size, the radius, the 64 in sight, the rates, the kinds' speeds and the
  bounds. Its defaults are the budgets below.
- `og world-server load --bots N --gate` measures a server against them
  before you rely on it.

| Budget | Value |
|---|---|
| down per client | ≤ 8 KiB/s at p95 |
| up per client | ≤ 1 KiB/s |
| in sight | 64 |
| per area (two cells around one) | 200 |
| per server, 4 vCPU | 2000, tick p99 ≤ 10 ms |

<a id="guide-shared-worlds-6"></a>
### `shared-worlds §6` Replay and determinism

- Remote avatars are world state: they are in the world hash, and every
  message that moves them is host input, applied at a tick's start and
  kept in og.platform.journal. `og replay record --platform-journal <f>`
  replays a session, web or native, tick for tick with no server.
- What the client sends is derived from the world, so a replay recomputes
  it and sends nothing.
- Nothing in a system may read the connection directly; read
  og.RemoteAvatar entities, the shared events and og.shared.status.
