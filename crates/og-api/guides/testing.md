# Building, checking and testing

The build, check and test loop of an Origin Engine game. Playtests as bun:test files, invariants first (og.assert, playtest invariants and require), frames only for looks, what an unchecked result means, the probe and dumps, and performance.

The API of everything here: `og api show assert`, `og api show commands`,
`og api show probe`, `og api show debug`, and `runPlaytest`, `PlaytestDesc`
and `PlaytestRun` in `@origingame/engine/playtest`. `og <command> --help`
gives each command's flags.

## 1. The loop

| Step | Command | When |
|---|---|---|
| type-check | `og check` | after every edit; `og run`, `og test` and `og build` run it first |
| run once | `og run --headless --commands "1:dump_world;2:quit"` | to see what the world holds |
| live | `og dev` | while iterating on looks, UI and feel: hot reload keeps the world |
| live on the web | `og dev --web` | the same in a browser: the page at `http://127.0.0.1:47810/` |
| test | `og test` | before claiming anything works |
| build | `og build --publish` | before handing the game over |

- `og dev` reports each reload against a 300 ms budget. A reload over it
  usually means a large `src/` file or an asset cooked on every save;
  split the file or move the asset's generation into the cook.
- `og dev --web` serves the development web build (the inspector on F1,
  Tune, Pause and Step, every probe member on `__runtime_probe__`) and
  pushes each save to the open page over `_dev/socket`: an asset or UI
  edit is cooked alone and applied in place, a `src/` edit bundles the
  game's own code and swaps it in with the world kept (snapshot, swap,
  restore), so a store's initial value or a `start` change shows only
  after a restart; only a `project.ogt` change reloads the page. A cook,
  bundle or type error stays on the page's console and in
  `_dev/status` (`{ready, build, error, last}`) until fixed; the server
  keeps running. `window.__og_dev_reloads__` lists each change applied
  with its steps, save to frame; the budget is 1.5 s for an asset or UI
  edit and 300 ms from the code bundle written to the frame.
- `og check` warnings are findings, not noise: the locale-font warning
  (screens §6) and the determinism warnings (determinism §1) each name their fix.
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

## 3. Frames are for looks

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

## 4. The probe and the dumps

`og run --port P` and `og dev` serve the probe; `og probe <method>` calls
it once. Read state before guessing at it:

- `dump_world`, `dump_entity:<name>`, `dump_ui`: what the world, an entity
  and the UI hold right now. Name entities: a dump of anonymous ids is
  hard to read and impossible to query by name.
- Something not on screen: `og probe visible '{"id": "door"}'` (or
  `dump_entity`'s `visible`) says whether the main camera draws it and
  why, as a reason code: `drawn`, `hlod`, `hidden` (with the `visible`
  field that hides it), `loading` (with the asset), `layer`,
  `outside_view`, `untracked` (it draws through particles, lines or text,
  which the cull record does not follow) or `nothing_to_draw`.
  `dump_drawn` lists that verdict for every entity with something to draw.
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
- `og run --profile out.md` samples the game's TypeScript while it runs
  and writes the hot functions by self time and the call tree, each with
  its `src/` file and line (`out.cpuprofile` instead opens in Chrome's and
  VS Code's profile views). Time inside the engine shows on the binding
  that called it. Run enough ticks for the samples to mean something: at
  one sample a millisecond, a system needs a few hundred ticks to show.
- First frame on a phone over 4G within 2 s for a small game: a large
  first scene, fonts for every language at once and uncompressed textures
  are the usual causes. Stream what the first screen does not show.

## 7. The command channel's checks and edits

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

## 8. Debug draw and debug views

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

## 9. Machine-readable output

A tool that drives `og` reads these shapes; each is pinned by a test, and
a change to one is a breaking change. Fields may be added; none is
renamed, retyped or removed without one.

**`og test --json`.** Stdout is JSON Lines, one object per line, and
nothing else; the prose, the bun tests' report and the summary go to
stderr. The exit status is 0 exactly when the `test` event's `ok` is true.
Each line has an `event`, in this order:

1. `check`, unless `--no-check`: `ok`, `tool`, `version`, `config`, `ms`,
   `diagnostics` (`{file, line, col, code, message}` each), `uncheckedCount`.
2. `playtest`, once for each `tests/*.og.ts` file that ran, in the files'
   order (none when the check failed): `playtest` (the file's name without
   `.og.ts`), `file`, `ok`, `status` (`passed`, `unchecked` or `failed`),
   `tests`, `checks` (every `expect()`), `frameChecks`, `unchecked` (the
   frames left unchecked), `uncheckedCount`, `failures` (strings),
   `failureCaptures` (`{run, world, snapshot, frame}` each), `out` (the
   file's output directory, `og-out/test/<playtest>/`), `ms`, `assertions`
   (`{violations, uncovered, required, defaults}`), `runs`, `adapterErrors`
   and, when a run happened, the first run's `tick`, `hash`, `hashes`,
   `tier`, `renderer`, `adapterError` and `warnings`.
3. `test`, last: `ok`, `suites`, `passed`, `failed`, `uncheckedSuites`,
   `uncheckedCount`, `skipped`, `renderer` (`null`, or `"none"` when a run
   found no GPU), `adapterError`, `adapterErrors` and `check` (`{ok, ms,
   errors}`, `{ok: false, error}` when the checker could not run, or
   `null`). `unchecked` is an old alias of `uncheckedCount`.

**`og probe <method> [params-json]`.** One call to the probe of a running
`og run --port P` or `og dev` (`og api show probe` lists the members and
their results). Stdout is the call's `result` as JSON and nothing else,
exit 0; a refusal (`false`, `null`) is a result. A JSON-RPC error prints
`og probe: <method>: error <code>: <message>` on stderr and exits 1; so
does a probe that cannot be reached. A `capture` result's `png` is the
path the PNG was written to (`--out`, default `capture.png`) instead of
base64. A `capture_views` result's frames are written beside it as
`<stem>-<view>.png` (`--out shots/a.png` writes `shots/a-0.png`,
`shots/a-1.png`, ...; by default `capture-0.png`, ...), and each frame
prints `{view, width, height, scale, png}` with `png` its path. `--raw`
prints the whole answer frame, `{jsonrpc, id, result}` or
`{jsonrpc, id, error}`, on one line.

**`og build --json`.** Stdout is JSON Lines: the `check` event as above,
then one `build` event: `ok`, `out` (the absolute directory written),
`publish`, `files` (`{path, bytes, compressed, codec}` each), `total`,
`downloads`, `probe` (`{build, members}`) and `manifest` (the build's
`og-build.json`). `codec` names what `compressed` measures: `brotli-11`
(brotli -q 11, what a host serves) for a `--publish` build, `brotli-5`
for a development build (measured in seconds rather than minutes, and
about 30% above -q 11 for the engine module), `gzip-9` where the brotli
CLI is missing. Judge download size on a publish build. A failed build prints its event with `ok: false` and a
`reason`, and exits 1. The play page is `<out>/<manifest.entry>`.

**Where outputs land.** A build is written to `--out`, else project.ogt's
`publish { out }`, else `<project>/dist/`. Everything else a command writes
goes under `<project>/og-out/`, which a build never reads: `og run`'s
screenshots and dumps (`--out`, default `og-out/`), a playtest's runs
(`og-out/test/<playtest>/run-<n>/`), `og render` (`og-out/render/`),
`og run --device` (`og-out/device/`), `og explore` (`og-out/explore/`),
`og ui check` (`og-out/ui-check/`) and `og ui preview` (`og-out/ui-preview/`).
