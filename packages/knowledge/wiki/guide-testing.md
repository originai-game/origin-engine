<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `testing` guide: Building, checking and testing

The build, check and test loop of an Origin Engine game. Playtests as bun:test files, invariants first (og.assert, playtest invariants and require), frames only for looks, what an unchecked result means, the probe and dumps, and performance.

The API of everything here: `og api show assert`, `og api show commands`,
`og api show probe`, `og api show debug`, and `runPlaytest`, `PlaytestDesc`
and `PlaytestRun` in `@origingame/engine/playtest`. `og <command> --help`
gives each command's flags.

<a id="guide-testing-1"></a>
### `testing §1` The loop

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
  (screens §6) and the determinism warnings (determinism §1) each name their fix.
- A failing `og test` is read from its first failure: the tick, the
  entity, the field and the value. Fix the cause in the game, never by
  loosening the invariant that caught it.

<a id="guide-testing-2"></a>
### `testing §2` Invariants first

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

```ts
og.assert.always(world.get(hero, Health).hp >= 0, "hp never negative", { hp });
og.assert.sometimes(boss.phase === 2, "boss reaches phase 2");
og.assert.reachable("door opened");
og.assert.unreachable("fell through the floor");
og.assert.each(level, cleared, "each level cleared"); // one bucket per distinct key
```

- An assertion is its name and the place it is called from
  (`src/player.ts:42`). The build records every `og.assert.<kind>(...)`
  site, so a site that never runs is still listed. At the end of a run,
  every `sometimes`, `reachable` and `each` bucket that never fired is
  uncovered: a sometimes that never fires is a test that never got there.
- Violations and coverage reach `og run --json`'s `assertions` and
  `og test`'s results through the probe (`assertions`): the tick, the name
  and place, and the values passed as details.
- A call that passes costs a lookup in the runtime, without a call into
  the engine.
- The default invariants run beside them in every development and test
  build: no NaN or infinity in a reflected float, every number inside its
  field's declared range, no Entity field naming a despawned entity, and
  every entity inside og.WorldBounds when the world declares them. Give
  fields real ranges and declare the bounds: each is a free check. They
  read the components that changed (where the world tracks their changes)
  or every row, never write, and name the entity, component, field and
  value of their first failure.
- A playtest's `invariants` (`"state:lives>=0"`,
  `"entity:hero:og.Transform.position.y>-5"`) check the script's own
  properties from tick 0, without game code. A value written `$<path>` is
  another field of the same source (`"state:coins<=$total"`); without the
  `$` it is the text `total`.
- Assertions cost nothing in publish builds and never touch the world
  hash: leave them in.
- `og run --json` prints each site's first firing and first violation as
  an `{"event":"assert", "outcome", "tick", ...}` line as it happens, before
  the summary; a tool stepping a run reads the probe's `assertionEvents`
  after each advance instead of the whole report.

A playtest file, `tests/<name>.og.ts`, is an ordinary `bun:test` file:
`describe`, `test` and `expect` as Bun and Vitest write them. Inside a test,
`await runPlaytest({ commands, ... })` plays the script headless in fresh
processes of the game `og test` built and resolves to the run: its tick,
hashes, files (`run.json`, `run.png`, `run.wav`), `run.then` relaunches and
`run.peers` in a room. Each call is a fresh world, so one test can play two
runs and compare them.

```ts
import { describe, expect, test } from "bun:test";
import { runPlaytest } from "@origingame/engine/playtest";

describe("the shop", () => {
  test("a completed purchase pays 100 gems once", async () => {
    const run = await runPlaytest({
      commands: "0.5:buy;2.5:get:Wallet:wallet.json;2.6:quit",
      require: ["the purchase is delivered"],
    });
    expect(run.json<{ value: { gems: number } }>("wallet.json").value.gems).toBe(100);
  });
});
```

- The script's `invariants`, `require` and `expect:` steps are one
  assertion: `runPlaytest` rejects with each failure, its tick and its
  value. A test can be the call alone.
- Every test makes an assertion; one that made none fails.
- Pick the matcher by meaning: `toBe` for a number, string or boolean,
  `toEqual` for a parsed dump or an array, `toStrictEqual` when an
  `undefined` member matters. A tolerance is absolute:
  `expect(Math.abs(a - b)).toBeLessThanOrEqual(0.1)`; `toBeCloseTo` takes
  decimal places, not a tolerance.
- `og test` runs each file in its own `bun test` against the built game;
  a bare `bun test` has no game to play.

Write one test per thing the game promises (the loop completes, the
player can lose, a save restores, a level can be cleared), each a few
seconds of ticks. Drive input with the command channel's actions (key,
pointer, touch, gamepad, `move_to_text`), not by setting state the player
could not set.

<a id="guide-testing-3"></a>
### `testing §3` Frames are for looks

A screenshot answers what the game looks like, never whether it works.

- Put every check that reads a frame inside `run.frame(files, () => ...)`.
  Without a renderer the frame is unchecked: the check does not run, the
  report says so, and the test goes on to its other checks. A frame read
  outside it without a renderer fails the test (FrameUnchecked).
- An unchecked frame is not a passed frame. `run.unchecked` lists them
  and `run.tier` is `none`. Never report a look as verified from a run
  whose tier was `none`; rerun where a GPU is present, or say that it was
  not checked.
- A reference frame is recorded only after the look was reviewed
  (art-review §4), and re-recorded only when a change is meant to move it.
- Frame captures draw the tick, not an interpolated moment, so two
  captures of one tick match.

<a id="guide-testing-4"></a>
### `testing §4` The probe and the dumps

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

<a id="guide-testing-5"></a>
### `testing §5` Done means

- `og check` clean, `og test` green, every `require` site fired, no
  violation, and the uncovered list read: each uncovered `sometimes` is a
  path no test reaches.
- `og ui check` read for every screen (screens §5) and its findings fixed.
- The look reviewed on frames from a run with a renderer (art-review §4).
- `og build --publish` succeeds; a publish build keeps only the
  player-facing probe members, so a feature that relies on a dump or a
  command works only in development.

<a id="guide-testing-6"></a>
### `testing §6` Performance

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

<a id="guide-testing-7"></a>
### `testing §7` The command channel's checks and edits

The command channel (`og run --commands "t:action[:args];..."`, a
playtest's `commands`) drives a run and checks it. `help` lists every
action; this section gives the forms of the ones that check and edit.

`expect:<what>` asserts at its tick and `invariant:<what>` after every tick
from its own on. Their forms:

| Form | Checks |
|---|---|
| `entity:<id>:within:x,y,w,h` | the entity's visible box, in CSS pixels |
| `entity:<id>:within:x0,y0,z0,x1,y1,z1` | its position, in world units |
| `event:<Name>[:count]` | the event was emitted (that many times) |
| `ui:text:[=]<text>` | the UI shows the text (exactly with `=`) |
| `region:x,y,w,h:color:#rrggbb[:tolerance]` | a screen region's colour (expect only) |
| `frame:<ref.png>:rmse:<=v` | the frame against a reference (expect only) |
| `state:<path>==<value>` | a field of the probe's state(); also `!=`, `<`, `<=`, `>`, `>=` |
| `entity:<id>:<path>==<value>` | a field of its dump_entity (`render.lod==2`, `og.Transform.position.y>0`) |
| `timings:<path>==<value>` | state().timings (`triangles<=1500000`) |

- A value written `$<path>` is another path of the same source, read when
  the expectation is evaluated: `state:coins<=$total`,
  `entity:hero:Health.hp<=$Health.max`.
- A failed `expect` is a warning with the tick and the observed value;
  `og run --json` lists every result and exits 1 on a failure. A failed
  `invariant` is an assertion violation (og.assert's report, the probe's
  `assertions`), which fails `og run --json` and the playtest. A
  playtest's `invariants` are invariant steps.
- Without a renderer, frame expectations, an entity's `render.*` fields
  and the renderer's timings (`triangles`, `render`, `passes`) are
  unchecked (§3).

`get:` reads through the schema as game code sees it: an enum as its
variant's name (`"music"`), og.Time whole with its rate and the time
scale, and the DirectionalLight og.DayNight drives with the colour,
intensity and direction it is drawn with. Each read is logged and
written as JSON to `get-<tick>.json` (more unnamed ones on one tick:
`get-<tick>-2.json`, ...).

Editing scenes from a run, after tuning with `set:` or the probe's patch:

- `save_scene` writes every scene the world was assembled from back from
  the world: the boot scene's, then each one World.instantiate made,
  each to its own `.ogt`. Only the lines whose values changed are
  rewritten (comments and order kept): the fields set or patch tuned, and
  resources as `resource <Name> { }` blocks. With `spawned`, the entities
  the game spawned at run time go under the boot scene's marked section.
  Without a path each scene goes to its source; with one, the boot scene
  goes to that file under the output directory and the others beside it.
- The command
  ```
  apply_prefab:<entity>[:<Component>[.<field>]][:base]
  ```
  moves an entity's prefab overrides (all, a component's
  or one field, a field set at run time too) into its prefab's `.ogt` at
  the entity's node and out of its scene's instance block, so the scene
  cooks to the same world and every other instance takes the values; with
  `base`, into a variant's base. `revert_prefab` drops them instead.

<a id="guide-testing-8"></a>
### `testing §8` Debug draw and debug views

- og.debug draws lines, boxes, spheres, arrows, axes and labels in the
  world, each for a number of ticks. They are drawn over the finished frame
  (after every post effect, before the UI), so they read in every capture
  and never hide behind geometry: draw a hitbox, a sight cone or a path
  while chasing a bug, then look at a capture.
- The command channel's `debug:` actions switch the debug views: physics,
  navigation, bounds, levels of detail, and the lit image's terms (shadow
  cascades, ambient occlusion, albedo, image-based and direct light). Turn
  one on before judging why a frame is dark or an agent stuck.
- Leave debug draws in: a publish build compiles every member to nothing
  that answers its default. Debug draw is rendering, never fed back into
  the simulation, and not world state, so a snapshot or a replay does not
  carry it.
