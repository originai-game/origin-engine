# Building, checking and testing

The build, check and test loop of an Origin Engine game. Invariants first (og.assert, playtest invariants and require), frames only for looks, what an unchecked result means, the probe and dumps, and performance.

The API of everything here: `og api show assert`, `og api show commands`,
`og api show probe`, `og api show debug`, and the `PlaytestDesc` type in
`@origingame/engine/playtest`. `og <command> --help` gives each command's
flags.

## 1. The loop

| Step | Command | When |
|---|---|---|
| type-check | `og check` | after every edit; `og run`, `og test` and `og build` run it first |
| run once | `og run --headless --commands "1:dump_world;2:quit"` | to see what the world holds |
| live | `og dev` | while iterating on looks, UI and feel: hot reload keeps the world |
| test | `og test` | before claiming anything works |
| build | `og build --publish` | before handing the game over |

- `og dev` reports each reload against a 300 ms budget. A reload over it
  usually means a large `src/` file or an asset cooked on every save;
  split the file or move the asset's generation into the cook.
- `og check` warnings are findings, not noise: the locale-font warning
  (screens §6) and the lockstep warnings (determinism §3) each name their fix.
- A failing `og test` is read from its first failure: the tick, the
  entity, the field and the value. Fix the cause in the game, never by
  loosening the invariant that caught it.

## 2. Invariants first

A game's correctness is stated as properties the engine checks every
tick, not as pictures compared afterwards.

- `og.assert.always(cond, name, details)` for what must hold on every
  call: health never negative, a score that only grows, a player inside
  the level. `og.assert.unreachable(name)` in the branch that must never
  run.
- `og.assert.sometimes`, `og.assert.reachable` and `og.assert.each` for
  what a run must get to: the boss's second phase, the door opening, each
  level cleared. A site that never fires is listed as uncovered, not
  failed; name it in the playtest's `require` to make it a failure.
- The default invariants run beside them in every development and test
  build: no NaN or infinity in a reflected float, every number inside its
  field's declared range, no Entity field naming a despawned entity, and
  every entity inside og.WorldBounds when the world declares them. Give
  fields real ranges and declare the bounds: each is a free check.
- A playtest's `invariants` (`"state:lives>=0"`,
  `"entity:hero:og.Transform.position.y>-5"`) check the script's own
  properties from tick 0, without game code.
- Assertions cost nothing in publish builds and never touch the world
  hash: leave them in.

A playtest is a script and its properties. Its `check` function is
optional when `invariants`, `require` or `expect:` steps say everything.
Write one per thing the game promises (the loop completes, the player can
lose, a save restores, a level can be cleared), each a few seconds of
ticks. Drive input with the command channel's actions (key, pointer,
touch, gamepad, `move_to_text`), not by setting state the player could
not set.

## 3. Frames are for looks

A screenshot answers what the game looks like, never whether it works.

- Put every check that reads a frame inside `t.frame(files, () => ...)`.
  Without a renderer the frame is unchecked: the check does not run, the
  report says so, and the playtest passes on its other checks.
- An unchecked frame is not a passed frame. `run.unchecked` lists them
  and `run.tier` is `none`. Never report a look as verified from a run
  whose tier was `none`; rerun where a GPU is present, or say that it was
  not checked.
- A reference frame is recorded only after the look was reviewed
  (art-review §4), and re-recorded only when a change is meant to move it.
- Frame captures draw the tick, not an interpolated moment, so two
  captures of one tick match.

## 4. The probe and the dumps

`og run --port P` and `og dev` serve the probe; `og probe <method>` calls
it once. Read state before guessing at it:

- `dump_world`, `dump_entity:<name>`, `dump_ui`: what the world, an entity
  and the UI hold right now. Name entities: a dump of anonymous ids is
  hard to read and impossible to query by name.
- `get:<Resource>.<field>` and `set:<Resource>.<field>=<value>` read and
  set reflected state in a script.
- `replay record` and `replay compare`: the first tick at which a run
  diverges from a recorded trace, with the world diff there. Use it when
  a change altered behaviour it should not have, and for any lockstep game.
- `og render --commands ... --out clip.mp4`: a video of a scripted run for
  a reviewer; its fps must divide the tick rate.

## 5. Done means

- `og check` clean, `og test` green, every `require` site fired, no
  violation, and the uncovered list read: each uncovered `sometimes` is a
  path no test reaches.
- `og ui check` read for every screen (screens §5) and its findings fixed.
- The look reviewed on frames from a run with a renderer (art-review §4).
- `og build --publish` succeeds; a publish build keeps only the
  player-facing probe members, so a feature that relies on a dump or a
  command works only in development.

## 6. Performance

- The frame budget is 16.6 ms at 60 Hz and 8.3 ms at 120 Hz on the target
  device, not on the development machine; a phone game is measured with
  `og run --device` on a phone or its closest stand-in.
- A TypeScript system over 10,000 entities should stay within 1 ms a tick
  on a current phone. A system that allocates per entity per tick (arrays,
  closures, objects in a query loop) is the usual reason it does not.
- The probe's timings (`ProbeTimings`) split CPU and GPU by pass. Find the
  pass that grew before changing anything.
- First frame on a phone over 4G within 2 s for a small game: a large
  first scene, fonts for every language at once and uncompressed textures
  are the usual causes. Stream what the first screen does not show.
