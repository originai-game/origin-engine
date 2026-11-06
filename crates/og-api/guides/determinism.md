# Determinism, physics and lockstep

The engine's rules a game must keep, determinism first, and where they bite: the world and systems, physics, destruction, and lockstep multiplayer over a room. Read before writing simulation code.

The API: `og api show world`, `og api show system`, `og api show systems`,
`og api show rng`, `og api show math`, `og api show reflect`,
`og api show time`, `og api show physics`, `og api show destruction`,
`og api show net`.

## 1. The rules

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
- Iteration order is the query's. Do not sort by object identity or rely
  on `Set` order of objects created at different times.
- Presentation never feeds back: nothing a system reads may come from
  what was drawn (a pose from og.motion, a UI element's measured box, a
  particle's position, the camera's interpolated frame). Gameplay that
  needs a position reads the simulated one.

`og test` with `runs: 2` (or more) replays the script in fresh processes
and lists every end hash: equal hashes are the check. Run it on any game
that saves, replays or plays online.

## 2. Physics

- Bodies move through the physics step, never by writing og.Transform of
  a dynamic body: a written transform teleports it, losing contacts and
  velocity. Move kinematic bodies by their target, dynamic ones by force,
  impulse or velocity.
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

## 3. Lockstep over a room

Every peer runs the whole world; only inputs and hashes cross the room.

- Exactly one system reads og.Input and calls og.net.submit; every other
  system reads og.PlayerInput (or the game's own input shape, declared
  with og.net.shape before the room starts, the same on every peer). An
  input read anywhere else is this peer's alone and parts the peers.
- Inputs act `inputDelay` ticks after they are submitted. Design for it:
  local feedback (a sound, a flash) may react at once from the
  presentation, the simulation may not.
- `og check` warns on every call that differs between peers (Math's
  approximated functions, `Math.random`, the clock). Fix each; mark a
  line that only draws with a `// lockstep-ok` comment.
- With rollback, systems run again for re-simulated ticks: effects that
  leave the world (sounds, particles, haptics) check og.net.resimulating
  or come from events the engine deduplicates.
- A peer that stalls past `stallTimeout` loses its seat (og.SeatDropped,
  on one tick for everyone); the game hands that seat to a bot or ends
  the match, the same way on every peer.
- og.Desync names the first divergent tick; `dump_desync` gives the
  component-level diff. Read it before guessing.
- Test with `PlaytestDesc.room`: several processes, distinct scripts per
  seat, and assertions on `run.lockstep` and the peers' `netHash`. Two
  peers fed the same inputs agreeing proves little; feed them different
  ones.
- `og relay` plus `og run --room ws://127.0.0.1:P/<room>` plays a room by
  hand.

## 4. Destruction

- Fracture happens in the cook, never at run time: a destructible model
  is cooked into pieces once, and breaking swaps in the pieces. Changing
  the fracture settings re-cooks; it never costs a frame.
- Debris is simulated: hundreds of live pieces cost physics time every
  tick. Let pieces sleep and despawn, and bound how many a level may
  break at once.
- Breaking is world state, so it replays and syncs over lockstep like
  any other change; dust and sparks are presentation and do not.
