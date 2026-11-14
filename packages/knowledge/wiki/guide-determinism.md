<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `determinism` guide: Determinism, physics and lockstep

The engine's rules a game must keep, determinism first, and where they bite: the world and systems, physics, destruction, and lockstep multiplayer over a room. Read before writing simulation code.

The API: `og api show world`, `og api show system`, `og api show systems`,
`og api show rng`, `og api show math`, `og api show reflect`,
`og api show time`, `og api show physics`, `og api show destruction`,
`og api show net`.

<a id="guide-determinism-1"></a>
### `determinism §1` The rules

The simulation is deterministic by construction: the same seed and the
same inputs give the same world hash on every machine, native and web.
Everything else rests on this: saves, replays, hot reload, playtests and
lockstep. A game keeps it by keeping these rules:

- Time comes from og.Time (tick, dt, elapsed), never `Date.now()`,
  `performance.now()` or timers. The tick is fixed (project.ogt's `tick`,
  60 by default); `dt` is constant. Never scale movement by a measured
  frame time.
- Randomness comes from the engine's seeded og.Rng, never `Math.random()`.
- Transcendental maths comes from `math` (`math.sin`, `math.atan2`,
  `math.pow` ...) exported by `@origingame/engine`, never `Math.sin` and
  its kin, whose last bits differ between browsers.
- State is in components and resources. Module variables, closures that
  capture state, and `Map`s outside the world are lost on restore and
  differ after a replay.
- Iteration order is the query's. A `Map` or a `Set` iterates in
  insertion order, the same on every machine only while the insertions
  are: one filled outside the world (rule above) or in the order events
  arrived from the host is not. Sort by a simulated value with a total
  order (`a < b ? -1 : a > b ? 1 : 0` on a number, a handle or a string),
  never by `localeCompare`, which follows the host's locale.
- Presentation never feeds back: nothing a system reads may come from
  what was drawn (a pose from og.motion, a UI element's measured box, a
  particle's position, the camera's interpolated frame). Gameplay that
  needs a position reads the simulated one.

`og check` reports where a game's sources use what these rules exclude:
`Math.random`, `crypto.randomUUID`, `Date.now()`, `performance.now()`,
`new Date()`, `setTimeout` and `setInterval`, Math's approximated
functions and `**` with a non-integer exponent, what leaves the game
without the engine (`fetch`, `WebSocket`, `localStorage`, `indexedDB`),
what orders or formats by the host's locale (`localeCompare`, the
`toLocale...` methods, `Intl`) and what observes the garbage collector
(`WeakRef`, `FinalizationRegistry`).
Nothing hides these from game code, so the report is the only notice. A
finding is a warning with its replacement and never fails the check. Fix
it wherever a system that changes the world runs the line; a line that
only draws (a wobble in a shader parameter, a clock on the HUD) keeps it
with an `// og-ok` comment. Saves, scores, rooms and anything on a
network go through og.platform; a file goes through og.load.

`og test` with `runs: 2` (or more) replays the script in fresh processes
and lists every end hash: equal hashes are the check. Run it on any game
that saves, replays or plays online.

<a id="guide-determinism-2"></a>
### `determinism §2` Physics

Physics simulation is world state: the fixed step, seeded inputs and
snapshotted solver keep moving bodies deterministic across machines.

- Move dynamic bodies by force, impulse or velocity. `applyForce` lasts
  one tick; apply it each tick for continuous acceleration. Writing
  og.Transform teleports a body rather than moving it through contacts.
- A kinematic body integrates og.Velocity each tick. A Transform write
  teleports it before integration; it is not Rapier's next-position
  target and does not infer velocity. A fixed body is not solver-driven.
- Units are metres, kilograms and seconds. A character about 1.8 m tall,
  gravity 9.81 m/s². Objects far outside 0.05 to 100 m behave badly:
  scale the game, not gravity.
- Fast, small bodies (bullets, balls over about 30 m/s) tunnel through
  thin colliders; enable continuous collision on them or use a ray or
  shape cast per tick instead of a body.
- Character controllers are not rigid bodies: they slide, step and snap
  by their own settings. Pick the controller for a player character and a
  body for things the player pushes.
- Queries (raycasts, overlaps) see the colliders as of the last step. A
  body spawned this tick is not hit until the next.
- 2D and 3D are separate worlds: a 2D project's publish build leaves the
  3D physics out and refuses 3D-only features (terrain, heightfields) by
  scene file and line.
- Large worlds: past a few kilometres f32 positions lose millimetres;
  declare og.WorldOrigin so the world shifts around its focus. Keep its
  `step` a power of two.

<a id="guide-determinism-3"></a>
### `determinism §3` Lockstep over a room

Every peer runs the whole world; only inputs and hashes cross the room.
These are the rules a game keeps to stay in step; choosing a net mode,
input delay, stalls, desyncs and testing a room are in `networking §1` to
`networking §7`.

- Exactly one system reads og.Input and calls og.net.submit; every other
  system reads og.PlayerInput (or the game's own input shape, declared
  with og.net.shape before the room starts, the same on every peer). An
  input read anywhere else is this peer's alone and parts the peers.
- Inputs act `inputDelayTicks` ticks after they are submitted. Design for it:
  local feedback (a sound, a flash) may react at once from the
  presentation, the simulation may not.
- The findings of §1's `og check` report part the peers here, not only a
  replay: fix each one a system that changes the world can reach; mark a
  line that only draws with `// og-ok` (`// lockstep-ok` reads the same).
- With rollback, systems run again for re-simulated ticks: effects that
  leave the world (sounds, particles, haptics) check og.net.resimulating
  or come from events the engine deduplicates.
- What happens to a dropped seat (a bot, the match's end) is decided the
  same way on every peer, from og.SeatDropped.

<a id="guide-determinism-4"></a>
### `determinism §4` Destruction

- Fracture happens in the cook, never at run time: a destructible model
  is cooked into pieces once, and breaking swaps in the pieces. Changing
  the fracture settings re-cooks; it never costs a frame.
- Debris is simulated: hundreds of live pieces cost physics time every
  tick. Let pieces sleep and despawn, and bound how many a level may
  break at once.
- Breaking is world state, so it replays and syncs over lockstep like
  any other change; dust and sparks are presentation and do not.
