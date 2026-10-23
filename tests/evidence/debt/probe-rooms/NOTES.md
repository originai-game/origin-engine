# Debt: probe-depth and rooms (branch `debt/probe-rooms`)

These are the debts that streams S5 "probe-depth" and S4 "rooms" left in
their NOTES ("Left out"). The branch starts at origin/main 50f1dafd, and
each item is committed on its own. I ran no gate, no `og test` lane, no
web-reference, and no wasm or release build. One exception: I ran
`og test templates/archetypes/arena` once, by mistake. Every later playtest
run went through `bun src/node/playtest-run.ts` directly.

## Commits

| Commit | Item |
|---|---|
| 1d13f2d8 | 1 and 2: trace spans where they ran, GPU pass timing |
| 2112ce5f | 3: the web trace |
| 4de41f02 | 4: scene write-back for every scene, the spawned entities and resources |
| 3d88ea50 | 5: opaque UI counts in describe's occlusion |
| 8e54f83c | 6: `cook --report` figures for every asset type, and their budgets |
| 7519142f | 7: room playtests take `then` and `quit` |
| 2eeeb863 | 7: the `?room=<id>` Play link, and the two-page web room test |
| 3f68f635 | 7: the arena's `og polish` findings acted on |

## What landed, and the test that fails on the old code

1. **Trace spans with real starts.**
   - Every span now carries its start as well as its length:
     - og-ecs `SystemInfo.start`;
     - og-render `PassTiming { name, start, ms }`, in the new `timing.rs`;
     - og-lanes per-job timing (`set_trace`, `take_trace`, `set_trace_clock`);
     - og-cook `OutputReport.start_ms` and `worker`.
   - The trace has these threads: 1 engine, 2 render, 3 load, 4 cook,
     5 gpu, lane workers at 16+n, and cook workers at 48+n.
   - `packages/engine/src/trace.ts` builds the document on both hosts.
     Its `checkTrace` fails a trace whose spans are laid end to end.
   - Test: `packages/engine/tests/trace.test.ts`, on the 2500-box pile
     with `OG_LANES=4`. The old trace fails the end-to-end check.
2. **GPU pass timing.**
   - og-render's `GpuTimer` uses `TIMESTAMP_QUERY` where the adapter has
     it: empty compute passes write timestamps around each pass.
   - The GPU times appear in `ProbeTimings.gpuPasses`, on the trace's gpu
     thread, and in `og run --json`.
   - Where the adapter lacks timestamps, `RenderCapabilities.gpuTimestamps`
     is false.
   - The numbers are in the next section.
3. **The web trace.**
   - The engine Worker buffers the trace (its `trace` call: start, stop,
     take). The wasm exports `og_web_trace` and `og_web_trace_take`.
   - The shell has `trace()` and `saveTrace()`, the play page reads
     `?trace`, and the probe has `trace` and `saveTrace`.
   - Test: `packages/engine/tests/web-trace.test.ts`. It drives the real
     Worker and shell, with the native host bound in place of the wasm.
4. **Scene write-back.**
   - `save_scene` writes every scene that `og.load.scene` instantiated
     back to its own `.ogt`.
   - With `:spawned`, the entities the game spawned at run time go under
     a marked section (`SPAWNED_MARK`), so a reader can tell them from
     authored ones.
   - og-cook adds a `resource <Name> { }` scene entry and
     `SceneDesc.resources`, applied when the scene spawns. Tuned resources
     are now written instead of reported as skipped.
   - The round trip (cook → change → save → cook) stays exact.
   - Tests: og-cook `tests/scene_write_back.rs` (a new case) and
     `packages/engine/tests/scene-write-back.test.ts`, which fails on the
     old code.
5. **UI in describe's occlusion.**
   - og-ui reports `UiElement.opaque` for elements with an opaque
     background, and the IDL adds `DescribedUi.opaque`.
   - describe hides the id pass's pixels under those rects before it
     counts visibility.
   - Test: `packages/engine/tests/describe-ui.test.ts`, on templates/game's
     tick-1 frame at 800×450. The crate's occlusion was 0 on the old code
     and is 0.909 now.
6. **`cook --report` for every asset.**
   - Each output type reports its own figures, read back through the
     runtime loaders:
     - a sound: seconds, channels, rate, chunks;
     - a font: glyphs, kerning pairs, pages;
     - a scene: entities, instances, components, resources;
     - a navmesh: tiles, polygons, vertices, cells;
     - a timeline: tracks, seconds;
     - strings: locale, keys;
     - styles, behaviours, tilesets, tilemaps, rigs, 2D rigs, flipbooks,
       shaders, post passes, emitters, looks, palettes, LUTs, environment
       maps, videos, terrains and stream packs: their own figures.
   - New budgets, checked by `og doctor --project`: `soundSeconds` 600,
     `videoSeconds` 300, `fontGlyphs` 8192, `navPolygons` 65536,
     `sceneEntities` 10000 and `emitterParticles` 16384.
   - Test: og-cook `tests/cook_report.rs` (the old report has no `sound`,
     `strings`, `timeline`, … keys).
   - Evidence: `cook-report-game.json`. I also checked `og doctor` by
     setting `fontGlyphs 300` and `soundSeconds 0`: it flagged NotoSansSC
     at 508 glyphs and both sounds.
7. **Rooms.**
   - **`then` and `quit` in room playtests.**
     - Root cause: a lockstep gate that restored the authority's snapshot
       shifted the joiner's scheduled steps by the jump. The steps then sat
       on each process's own ticks, not the room's.
     - Fix: og-probe's `held` no longer shifts the schedule, so steps stay
       on the shared ticks. A step the jump passed runs at the next tick.
       A game's own restore still shifts the schedule (`shift_schedule`,
       which the after-tick hook uses).
     - `playtest-run.ts` relaunches every place for a `then` step, into a
       room named `<name>.then-<n>`. `room.ticks` is now optional, and each
       place gets its own store.
     - The arena's duel now ends by `quit` and compares its two reads
       exactly.
     - Tests:
       - og-probe `a_gate_that_restores_a_peers_snapshot_keeps_the_steps_on_the_shared_ticks`.
         On the old code the quit lands at tick 90 instead of 60.
       - `rooms.test.ts` "a room playtest's quit and its then step…". On
         the old code the two places quit at 240 and 238, and at 150 and
         148.
   - **The `?room=<id>` Play link.**
     - `shell/page.ts` reads `?room=` and passes it to
       `Shell.setRooms(host, link)`.
     - The engine joins the room itself, once, as soon as a host carries
       rooms (`og_room_set_host`'s new `join`). The game hears
       `og.RoomJoined` as it would for its own `roomJoin`.
     - The IDL adds `RoomInfo.link`. With no host, `og.RoomJoined` comes
       with ok false and says why.
     - The Worker's `advance` and `frame` calls now pump the room. Its new
       `net` call returns `og.net.status`, the shared hash and the room
       info.
   - **The two-page web test.**
     - `tests/web-rooms.test.ts` runs two pages, each a real Worker and
       shell in its own process (`tests/support/room-page.ts`) on the native
       host.
     - Each page's Player stand-in speaks `room.*` to an `og relay` that
       the test starts, and each joins by its Play link alone.
     - At tick 300 both pages are in lockstep, seated 0 and 1, with no
       desync and equal `netHash` (b5ad395284c68667 in the manual run).
     - On the old shell, which ignores the link, the test fails: the pages
       never join.
   - **`og polish` on the arena.**
     - Before: taste n/a, feel 42. The duel showed no feedback at all, and
       no playtest framed the HUD.
     - The fixes:
       - Coin, dash and round-won feedback (chime, sparks, "+1" in the
         taker's ink, hit stop, shake, whoosh, fanfare), played by the
         `Feedback` and `Voices` systems from the world alone, so every
         peer plays the same.
       - The neon UI style through `looks/arena.ogt`, replacing the
         prototype placeholder. The now-unused `fonts/ui.ttf` was removed.
       - A `tests/solo.og.ts` playtest (keys and `expect` steps) that
         frames the HUD.
     - After: taste 100, feel 66. Contrast, text size and HUD occupancy
       are 100, input latency 100 and idle 100.
     - What is left:
       - Frame time (lavapipe's software rasteriser takes about 27 ms to
         draw four sprites).
       - Sound on input is 50, because walking has no sound, by choice.
     - Report: `arena-polish.md`.

## Trace and timing numbers (lavapipe, tier vulkan-cpu)

templates/game, 300 ticks: 12917 spans, 0 `checkTrace` errors
(`trace-game-300.json.gz`).

| pass | CPU encode, ms | GPU, ms |
|---|---|---|
| cascade.0 | 0.02 | 2.05 |
| cascade.1 | 0.03 | 4.01 |
| cascade.2 | 0.014 | 3.79 |
| main | 0.047 | 108.6 |
| tonemap | 0.029 | 7.83 |

The 2500-box pile with `OG_LANES=4` (`trace-pile-lanes.json.gz`):

- GPU: main 133 ms, taa 8.4 ms, tonemap 8.5 ms.
- Lane jobs: physics 0.46 ms and cull 0.05 ms, spread over 4 lane
  threads.

## Frames read on lavapipe

- templates/game's tick-1 describe frame at 800×450 (item 5). The HUD
  panel spans x 25–251 and hides the crate except a strip at x 251–260.
- The arena solo frame at tick 120 (`arena-solo.png`): "Red 1" and
  "Blue 0" in the neon style at the top, the red and blue fighters, the
  coin, and "Local: Red" at the bottom. All of it is legible.

## Shared files touched

- **IDL and generated outputs:**
  - `crates/og-api/api.ogt` (ProbeTimings.gpuPasses,
    RenderCapabilities.gpuTimestamps, SavedSceneFile and saveScene's
    `scenes`/`spawned`, DescribedUi.opaque, RoomInfo.link) and the
    generated outputs (`crates/og-api/src/generated`,
    `packages/engine/types`, `schema`, `src/generated`, the wiki).
  - `crates/og-idl/src/gen/host.rs` (the docs of `og_room_set_host`).
- **og-render:** `lib.rs` (the clock fields, the TIMESTAMP_QUERY feature
  request, the pass loop's timing and the submit) and the new `timing.rs`.
  Only timing code.
- **og-ecs:** `schedule.rs`, `world.rs`.
- **og-lanes:** `lib.rs`.
- **og-cook:**
  - `cook/mod.rs`, `cook/scene.rs`, `cook/stats.rs`, `runtime.rs`,
    `scene_text.rs`.
  - Not the geometry code. Overlap with debt/geometry-video is possible in
    `runtime.rs` and `cook/mod.rs`.
- **og-probe:** `probe.rs` (the trace hooks and `held`), `hooks.rs`,
  `describe.rs`, `expect.rs`, `rpc.rs`, `command.rs`, `lib.rs`, and the
  conformance, expect and commands tests.
- **og-ui:** `ui.rs` (`UiElement.opaque`).
- **og-runtime:** `trace.rs`, `engine.rs`, `extract.rs`, `api.rs`,
  `probe.rs`, `assets.rs`, `scene_save.rs`, `describe.rs`, `net.rs`.
- **Hosts:** og-host-node `lib.rs` and `host.rs`; og-host-web `lib.rs` and
  `host.rs`.
- **og-cli:** `trace.rs`, `doctor_project.rs`.
- **packages/engine:**
  - `src/node/run.ts`, `src/node/playtest-run.ts`, `src/playtest.ts`,
    `src/index.ts`, `src/runtime.ts`, `src/trace.ts`.
  - `src/web/worker.ts`, `src/web/shell.ts`, `shell/page.ts`.
  - `tests/fixtures/web-glue/native.ts`.
- **templates/archetypes/arena:** the game, its look, sounds, emitter and
  tests.
- **Docs:** `docs/design.md`, `docs/web-export.md`,
  `docs/platform-asks.md` (#16's status), and
  `packages/knowledge/skills/og-game/SKILL.md`.

## Metal references owed

None. No reference frame was changed or added:

- The GPU timer adds only empty compute passes.
- describe's id pass is not a reference.
- The arena has no `references/`.

## Left out, and for the coordinator

- Rollback and the platform's side of ask #16 were out of scope. The
  engine reads `?room=`; the platform still has to put it on the Play link
  and answer `room.*`.
- No real browser ran the web paths (trace and rooms). Both tests use the
  real Worker and shell on the native host. The wasm exports were checked
  with `cargo check -p og-host-web --target wasm32-unknown-unknown` only;
  the wave end's web-reference should confirm them.
- The arena's remaining polish findings:
  - Frame time is lavapipe's rasteriser, not the game.
  - Walking has no sound (sound on input 50), by choice.
- **Decision for the coordinator.** A lockstep gate's restore no longer
  moves scheduled steps, while a game's own restore still does. A script
  that relied on the old shift for a joiner (none in the repository) would
  now see its steps on the shared ticks.
