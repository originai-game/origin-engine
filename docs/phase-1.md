# Phase 1 — execution plan

Scope: `docs/design.md` §11 phase 1 ("Core"), 13 weeks from 2026-10-20.
Output: a game author can `npm create @origingame/game`, write TypeScript
against `@origingame/engine`, run it in a browser and headless with `og`, and
the platform can run it in an `engine`-class sandbox. Spike findings
(`docs/spike-results.md`) are the baseline; the spike crate is the seed
of `og-render` and is deleted once its code has moved.

## Phase 3, lanes round (2026-10-19)

Branch `p3-lanes`, 211fc40..HEAD. Parallel systems (docs/design.md §2.2)
and the §7 reference scene measured with lanes off and on. Evidence:
`tests/evidence/P3/lanes/` (commands, machine load and tables in its
`README.md`).

- **Lanes** (100419e, b22e3e5, 8b91954): `crates/og-lanes` wraps rayon
  (`map` and `for_each_mut`, both in input order, `install` for a rayon
  library) behind one switch, `OG_LANES` (`off`, `0` or `1` serial, `N`
  threads, unset the cores up to 8) and the probe's `lanes:<on|off>`;
  `dump_lanes` and `og_web_lanes` report threads, pool and the parallel
  calls that ran. On lanes: og-anim's graph evaluation and palettes (64
  skeletons a task), extraction's row decoding (512 rows a task), og-render's
  frustum cull (1024 bounds a task; scheduling only, recording stays on the
  engine thread) and rapier's solver natively. A call splits only from two
  tasks' worth: waking the pool cost 0.5 to 2.6 ms a call on the loaded M4
  Pro against 21 us for the reference scene's whole animation pass, so the
  §7 scene stays serial everywhere but the native physics step. With lanes
  off a rayon library runs inline on the calling thread (`use_current_thread`;
  hop p50 1 us, was 110 us). TS systems keep their order on the engine
  thread.
- **Same world, lanes on or off** (a39d4f8, a236969, 6fcfea8): the gate's
  determinism step runs the host release with lanes off and on four threads
  and fails when the second ran nothing in parallel;
  `packages/engine/tests/lanes.test.ts` runs the reference scene plus a
  crowd (2000 boxes, 160 walkers, 48 crates) in two processes and finds the
  same world hash and captured pixels at ticks 60, 180 and 300 and the same
  replay with lanes switched mid-run. Results in completion order fail it.
- **The lanes wasm** (b09df7f, 2728f9c): two web modules, decided and
  documented in `docs/web-export.md` §Lanes. An atomics module cannot load
  on a page that is not cross-origin isolated, so `origin-engine_bg.wasm`
  stays single-threaded and `og build --publish` (isolation not `off`) also
  ships `origin-engine-lanes_bg.wasm` (`scripts/wasm-lanes.sh`: `-Z
  build-std` with `RUSTC_BOOTSTRAP=1`, its own target dir; docs/ci.md §The
  lanes wasm). The shell takes it when the page is `crossOriginIsolated`
  with more than 2 cores (`min(cores - 1, 8)` Workers through
  wasm-bindgen-rayon) and otherwise, or when the pool fails, boots the
  single-threaded module with the same API; the mode is on `ready.lanes`,
  the log, `__runtime_probe__.lanes` and `call("lanes")`. The service
  worker precaches the variant it predicts. COOP/COEP stay per publication
  (`_headers`, `origingame.runtime.json`).
- **Wasm** (the gate's size-gate at 1f56ad7): publish 3,153,927 B
  brotli, lanes 3,156,913 B (75.3 % of the 4 MiB budget, which the size
  gate now holds per file; the round grew the single-threaded module by
  about 4 KB).
- **Frame times** (e30fa0a, `bench/lanes.ts`, release, 1280x720, 600
  frames, M4 Pro at load 27 to 35 natively and up to 110 in Chrome), p50 /
  p95 / p99 ms:

  | run | lanes off | lanes on |
  |---|---|---|
  | Metal, reference scene | 6.968 / 14.668 / 17.598 | 6.475 / 9.031 / 10.329 (8 threads, physics only) |
  | Metal, crowd | 6.063 / 9.528 / 10.413 | 5.808 / 6.717 / 7.831 (8 threads, 5940 parallel calls) |
  | Chrome WebGPU, not isolated | 1.2 / 2.3 / 3.1 | (single-threaded) |
  | Chrome WebGPU, isolated | 1.19 / 2.235 / 3.225 | 1.265 / 2.305 / 3.16 (8 Workers, 0 parallel calls) |

  Each native pair ends in the same world hash and the three Chrome runs
  capture the same frame at tick 660. **§7 budget (16.7 ms)**: p95 held
  in every run and p99 in all but one, native lanes off at 17.598 ms, the
  GPU wait under host load (CPU p99 3.8 ms); single frames went over (max
  20.6 ms native, 33.1 ms in Chrome not isolated); earlier runs at load
  75 to 220 reached 23 to 59 ms at p95/p99 in both modes, so it should be
  repeated on a quiet machine. In Chrome lanes change nothing for this
  scene: nothing crosses the grains and physics is serial on the web. The
  Three.js comparison is an owner measurement and was not done.
- **Gate**: not completed on this branch. `OG_GATE_JOBS=2 scripts/check.sh
  --all` at 1f56ad7, 2026-10-19, M4 Pro at load 127 to 237: fmt, clippy,
  drift, wasm-build, test (Rust workspace), wasm-opt, size-gate and
  determinism (lanes on: 4 threads, 1200 parallel calls, every build
  agrees) passed; bun-test was stopped from outside mid-run (exit 143,
  the gates moved to main after merge), with two engine tests over their
  5 s timeout at that load (asset-stream, render trails) and
  web-export.test.ts unfinished. Earlier, `scripts/check.sh quick` at
  e30fa0a passed fmt, clippy, drift and bun-quick and failed test-quick on
  og-audio's wall-clock mix bound (1.7 s against 1 s in debug at load 55
  to 122; og-audio is untouched here). `bun test tests/lanes.test.ts`
  passes and fails with `og_lanes::map` broken to completion order.
- **Pre-existing**: `tests/web/export.ts --webgl` fails on base 211fc40 too
  (overlay.wgsl `textureNumLevels` on WebGL2, from the UI round's 42d7294;
  fixed on p2-showcase).
- **Left**: `og run --device` (another branch this round) must serve a
  build with its `_headers` so a phone gets lanes; the platform's pin bump
  and ask 11 (`docs/platform-asks.md`: keep COOP/COEP on lanes releases,
  allow `blob:` Workers, check `__runtime_probe__.lanes` on Play); rapier
  in parallel on the web (its `parallel` feature in the lanes wasm);
  a physics world too small to split still pays the pool hop natively with
  lanes on (no body threshold); the crowd has no Chrome measurement; whether
  the lanes wasm's 0.07 ms a frame over the single-threaded one (mean, same
  serial work) is noise or the atomics build is not separated.

## Status 2026-10-19, Phase 2, showcase round

Branch `p2-showcase` from main at b2532cd. The phase 2 deliverable "one
mid-size 3D showcase" is `templates/archetypes/third-person`, **Ember
Vale** (1253988, assets 20dc944, `create --archetype third-person`
314e252): a lamplighter on og.CharacterController walks, runs, wades and
jumps over a 128 m terrain vale with an og.Water lake; holding F by a
beacon walks it to the plinth and an og.IkChain reaches its right hand into
the bowl until the beacon catches (fire, smoke and ember particles, a
light, a scorch decal and a rune that glows, bloom, AO, god rays, the LUT
and grain under og.DayNight as the run's clock). Three sentinels on
og.NavAgent think with og.ai behaviours and og.Perception: patrol, alarm,
chase, grab (a heart, a hit stop, a shake). Both characters play an
og.AnimationGraph (a 1D idle, walk and run blend by speed, the hero's air
state). Four beacons before night win; the hearts or night lose; the
summary, Again or Title. Screens are og.ui on the fantasy pack's tokens
with MSDF text (Cinzel, Newsreader, a Noto Sans SC subset), English and
简体中文, touch controls on a phone; every `Tuning` field is Tune's; a
voice per event with ducked music; settings and a best-run record go
through `og.platform` (the win merges against a stale write from another
device). The publish build plays behind the tap gate. Only the public
`@origingame/engine` API; the characters, clips, textures, LUT and
emitters are the engine's own CC0 assets from fx-showcase, the rest is
written by `tools/make-assets.py` (no download; not trickster.glb or
dropship.glb). It typechecks in the gate through
`packages/engine/tsconfig.templates.json`'s `templates/archetypes/*/src`
glob (row 12).

### Evidence

- **metal**: `og test templates/archetypes/third-person` passes 8 suites
  (flow, frames, lose, pause, phone, replay, sentinels, win; 109 checks)
  through real input; frames title, beacon and dawn at 0.000 % against
  `references/metal/`.
- **webgpu and webgl2**: `tests/web/reference.ts --project
  templates/archetypes/third-person` (Chrome for Testing, the release
  `dist/web`) matches the same three frames against `references/webgpu/`
  and `references/webgl2/` at 0.000 %. Against the metal references:
  webgpu RMSE 0.00131 (title), 0.00475 with 0.060 % outliers (beacon),
  0.00067 (dawn); webgl2 RMSE 0.00428, 0.0181 and 0.00180. Frame read of
  the webgl2 beacon, the one over the harness's 0.01 against metal: the
  same scene, the hand in the bowl, fire, smoke and light alike; WebGL2
  draws the ember sparks as longer, wider streaks (about 1 % of the
  pixels), so it keeps its own reference.
- **vulkan-cpu**: not recorded (the gate does not run templates on it).
- **og polish** (`tests/evidence/P2/showcase/`, metal, release addon):
  taste 95, feel 85. Seven findings, recorded and not fixed: six labels
  under 4.5:1 on the fantasy pack's shared palette (Set out 3.36:1, the
  footer 3.5, Resume 3.84, Again 3.99, 完成 4.41, 语言 4.43), and on the
  phone 2 of 3 presses without a ui-bus sound (the HUD's Jump and Kindle
  answer on sfx). Frame time 67: the phone playtest at 1170×2532 (render
  p95 14.6 ms) under a load average of 20–30; 6.8 ms p95 or less at
  1280×720.
- **Wasm**: the publish wasm is **3,176,490 B brotli, 75.7 % of
  4,194,304 B** after merging main at 44d12a8 (3,061,324 B, 73.0 %,
  before it, against main's 3,061,522 B then): the game adds nothing to the
  engine; the growth is main's (3D SPH and the rounds before it).

### Engine fixes

- **og-nav** (d947221): a bake panicked on a triangle wholly above the
  bake box (the vale's cliffs over the navmesh bounds); such cells are
  skipped like those below it, with a test.
- This round also found and fixed the deferred start that the probe's
  `hello` broke since d2bd748 (f58417a) and the WebGL2 glass panic since
  42d7294 (c5ab008). Main fixed both the same day (4c2079a, merged in
  cf6aeaf, with `runtime.test.ts` and `crates/og-render/tests/webgl2.rs`);
  the merge f728a2a keeps main's code and tests and drops this branch's
  variants and its `core.test.ts` `defer` case. Ember Vale's metal
  references were recorded with the start after the renderer attaches, as
  main's fix also does: after the merge they pass at 0.000 % outliers
  (RMSE at most 0.0004, main's other render changes), not re-recorded.

A documentation gap, left as is: an og.IkChain target in `"world"` space
is in the frame of the skeleton body's Transform parent (here the hero's
unrotated capsule), not true world; `src/hero.ts` subtracts the hero's
position.

### Checks

On f728a2a (the merge of main 44d12a8), 2026-10-19, load average 50 to
220: `OG_GATE_JOBS=2 scripts/check.sh` **PASSED, 9 of 9 in 830 s** (fmt
2 s, clippy 17 s, wasm-build 2 s, test 190 s with 551 tests, wasm-opt
135 s, size-gate 42 s, drift 2 s, bun-test 199 s with 109 engine tests,
determinism 630 s). `og test` on metal with the release addon:
third-person 8 suites, idle 10, rhythm 11, roguelike 9, tower-defense 9
(the four failed on this branch before the merge and pass with main's
re-recorded frames, 7a5e320). `scripts/check.sh --only web-reference`
was stopped by the coordinator partway: fx-showcase's webgpu frames and
the 18 E5 stations (fluids3d among them) on WebGPU passed; on WebGL2
cloth, controller and fluid passed before it stopped, the rest of WebGL2
and the og-tests gfx scenes did not run. Ember Vale's own webgpu and
webgl2 references were last verified (0.000 %) before the merge, on
484f2e0. Before the merge, on 484f2e0, `--all` passed every step but a
bun-test load timeout (rerun alone, passed) and web-reference passed in
384 s.

### Not done

Owed rows 47–49: vulkan-cpu references for Ember Vale, the polish findings
above (the fantasy pack's palette is shared by every game that uses it;
the HUD buttons' bus is the template's), the IkChain space note in the
IDL's docs. Unverified after the merge: the rest of web-reference's
WebGL2 run, Ember Vale on webgpu and webgl2, and `og test` of ui-kit,
fx-showcase, runner, match3, topdown and game (all passed before it).

## Phase 3, culling and device round (2026-10-19)

Branch `p3-culling`, off 211fc40, with main merged in at cf6aeaf (acaf7e2).
The round covered GPU-driven culling, `og run --device`, and the shore foam
the streaming round left. Evidence is in `tests/evidence/P3/culling/`,
`tests/evidence/P3/device/` and `tests/evidence/P3/streaming/foam/`.

- **GPU-driven culling** (7d845e1, 53cdc86): on an adapter with compute and
  indirect draws (native Metal, Vulkan and DX12, and the browser's WebGPU),
  og-render culls every mesh pass in compute: the camera, the planar
  reflection, the cascades and the local shadow faces.
  - The visible instances are compacted in sorted order by a deterministic
    prefix sum, and each batch is one `draw_indexed_indirect`.
  - The camera pass also drops instances whose box lies behind a depth
    pyramid. The pyramid is built from last frame's visible occluders, drawn
    at this frame's camera.
  - WebGL2, `Culling::Cpu`, object motion blur and frames over the
    storage-binding limit keep the CPU path.
  - `FrameStats` gains `gpu_culling`, `occluded`, `occluder_draws` and
    `counts_lag`, and `og-gfx` takes `OG_GFX_CULLING=cpu`.
  - `crates/og-tests/tests/culling.rs` renders eight scenes both ways over
    several frames. It checks for the same picture, the same frustum counts
    and never more drawn.
  - On lavapipe, all 47 og-gfx scenes then present (48 with main's fluid-3d, which the merged gate covers) compare with their references exactly
    as the CPU path does. That needed one fix first, 53cdc86: llvmpipe
    reports level 0's size for every pyramid level, so each level's size is
    now computed from the screen's.
- **Culling review fixes** (branch `fix-culling`):
  - `Culling::Auto` takes the GPU path only from
    `GPU_CULLING_MIN_INSTANCES` (16,384) instances, from the measurements
    below: about 1.1 ms saved at 40,000 instances, 0.4 to 1 ms lost on
    scenes of tens. `Culling::Gpu` forces the GPU path, and `og-gfx` takes
    `OG_GFX_CULLING=gpu|cpu|auto`. The 47 gfx scenes are small, so under
    `Auto` they now cull on the CPU; `culling.rs` covers the GPU path with
    `Culling::Gpu`.
  - A frame with no occluder material, or no earlier camera visibility
    (the first frame, after the scan buffer grew), skips the occluders'
    pass, their depth and the pyramid, and culls in one compute pass.
  - The counts are copied back only after
    `Renderer::request_culling_counts` (og-tests-web asks every frame;
    natively `finish_stats` copies the last frame's late).
  - The buffers are sized for the frame's passes; a readback ring buffer
    in flight is kept when they regrow.
  - `culling.rs` compares the two paths at channel 2, 0.01 % outliers and
    RMSE 0.002, checks `gpu_culling` on every frame, and checks that the
    triangles drawn plus those hidden (`FrameStats::occluded_triangles`,
    counted on the GPU per hidden instance) cover the CPU's
    `in_view_triangles`.
- **Culling measured** (1f94224, b026d89, 757eca0) on og-gfx's perf-only
  city scenes, 40,000 instanced buildings at 1280x720.
  - **Metal**: over 40 interleaved rounds, `Renderer::render` takes 5.27 ms
    against 6.28 ms before on city-street, a paired ratio of 0.81. On
    city-aerial it takes 5.13 against 5.09 (0.97). Against the same binary
    culling on the CPU, the ratios are 0.83 and 0.86.
  - On the street, occlusion leaves 8,150 of 129,938 triangles.
  - Process CPU time per frame (user+sys) does not separate the paths beyond
    the noise: 3.0 to 3.6 ms for every variant.
  - The CPU saving is modest because both paths share the larger work:
    sorting, the instance upload, the bounds and encoding. The GPU saving is
    in the triangles.
  - **Chrome**: WebGPU with GPU culling takes 0.94 to 1.26 ms per frame,
    against 1.55 to 1.81 ms culling on the CPU, about 35 % less. WebGL2
    takes 1.84 to 2.27 ms.
  - `reference-gfx.ts` gains `--cpu-culling` and `--perf-out`, and times the
    city scenes when `--scene` names them.
  - Every native figure was measured on a machine shared at load averages
    of 25 to 200, so read the medians and ratios, not single runs
    (`culling/NOTES.md`).
- **`og run --device chrome|android[:serial]|iphone[:udid]`** (c8a0b9f,
  05e2df6) cooks and serves the publish build with COOP/COEP. It opens the
  page in Chrome over the DevTools protocol (Android through adb) or in
  Safari through safaridriver, taps the gate, and times the first frame and
  the played frames. It writes `og-out/device/{report.md,device.json}` in
  og polish's shape.
  - Chrome on WebGPU: first frame 270 ms, display p95 9.82 ms at 120 Hz.
  - Chrome on WebGL2, run again after main's glass fix: first frame
    1125 ms, engine work p95 1.41 ms.
  - `packages/engine/tests/web/device.ts` passes 27 of 27 checks.
  - The phone paths are written, and their prerequisites documented, but
    they are not exercised (owed row 40 remains the owner's).
- **Shore foam on vulkan-cpu** (ec54c60): large water now draws in cells of
  at most 256 m (`MAX_CELL`), so `terrain-far`'s 24 km sea is 94 x 94
  cells, not one quad. llvmpipe draws the foam line and the lake again, and
  Metal's foam band narrows to the same thin line. Metal against vulkan-cpu
  goes from RMSE 0.01771 to 0.00534. The sin/cos hypothesis was refuted,
  and `water.wgsl` is unchanged.
  - Re-recorded: `metal/terrain-far.png`, where the band is now a thin
    line and the old reference fails at 0.01561.
  - Re-recorded: `vulkan-cpu/terrain-far.png`, where the foam line and the
    lake edge appear (0.00944).
  - No other reference moved.
- **web-reference** (`scripts/check.sh --only web-reference`, after the
  render changes): PASSED on both tiers. That covers fx-showcase's
  stations, the E5 stations and og-tests' 47 gfx scenes, with GPU culling
  on WebGPU.
- **Gate** (`OG_GATE_JOBS=2 scripts/check.sh --all` on 6a99ead, at load
  averages of 150 to 300): fmt, clippy, wasm-build, test (535 passed, 9
  skipped), wasm-opt, size-gate and drift passed. bun-test failed on three
  tests over bun's 5 s default timeout. A rerun failed on one, the trails
  test (5.6 s). With `--timeout 60000`, all eight render tests pass in
  15.4 s. Determinism did not run, because the gate stops at bun-test. The
  final gate is run on main after the merge.
  - On the debug addon, the GPU path makes render.test.ts's small
    multi-renderer tests about 30 % slower: trails 1.8 s against 1.4 s with
    `Culling::Cpu`. Release og-gfx shows the same on small scenes:
    point-spot-lights 1.0 to 1.6 ms per frame against 0.6 ms, and showcase
    3.5 to 3.7 ms against 3.15 ms.
  - Pipeline creation is not the cost (`GpuCulling::new` takes 2 to 5 ms).
    The overhead is per frame, and below some instance count the CPU path
    is cheaper.
- **Wasm**: the publish build is 3,168,670 B brotli (75 % of the 4 MiB
  budget). That is +103,727 B over the streaming round's 3,064,943 B, for
  the cull shaders and pipelines, the depth pyramid and `--device`'s
  host-side code.
- **Left**:
  - Natively, GPU culling's CPU saving is about a fifth on an occluded view
    and about nothing on an open one, until the shared per-instance work
    (sorting and the whole-buffer upload; dirty tracking is still owed)
    shrinks.
  - In a browser loop that never yields, the GPU counts arrive late
    (`counts_lag` 237 in the timing loop). The frame is unaffected.
  - The phone paths of `og run --device` have not been run on a phone.
  - The 16,384-instance threshold comes from the round's measurements,
    taken before the review fixes cut the GPU path's fixed per-frame cost;
    it has not been measured again.
  - `vulkan-cpu` `terrain-grass` fails at RMSE 0.01488, on main as well as
    here. It predates this round.

## Phase 3, streaming round (2026-10-19)

Branch `p3-streaming`, 932dc00..HEAD. Terrain streaming, asset streaming
groups and the phase 2 terrain round's leftovers. Evidence:
`tests/evidence/P3/streaming/` (break-and-restore notes in
`break-and-restore.md`).

- **Terrain placement** (9941ffd, 6b9221b): og-render draws a terrain
  through a transform, and extraction hands it the entity's world matrix;
  og.terrain.sample, bounds and edit take the entity's rotation and scale.
- **Terrain tiles** (8c0e8bf, bc5c5ea, 8bc9174): og-render draws a
  terrain from a pyramid of 128-quad tiles in texture arrays, choosing
  them with `terrain_tiles_wanted` within `TerrainLod::budget` (16 MiB a
  terrain by default), each after its parent with a fade. og cook writes
  the pyramid (og.Terrain schema 2, every tile zlib'd on its own), and
  og-runtime's streamer decodes the wanted tiles off the frame (a worker
  thread natively; the browser's DecompressionStream on the web, the
  engine finishing at most 2 ms of tiles a frame), keeps twice the budget
  decoded and hands the renderer at most eight new tiles a frame. A
  capture settles first. The tick reads terrains through physics and
  og.terrain.sample only, never through what is resident.
- **Streaming and the world hash** (27e2f62): the probe's
  `terrain_stream:<stream|resident>[:<MiB>]` and dump_stream's `terrain`
  section; `packages/engine/tests/terrain-stream.test.ts` runs the
  fx-showcase terrain station 600 ticks resident and streamed at 0.25 MiB
  and finds the same world hash on every tick
  (`terrain/determinism.json`).
- **Frame times** (451f376, 9c7c986): `bench/terrain-fly.ts` flies the
  2 km island and a new 8 km terrain (4097², ridged) at 60 m on the native
  host, timing engine.frame plus the GPU wait (`og_node_wait_gpu`). On the
  M4 Pro at 1280x720 every run's p99 is 2.1 to 2.6 ms and no frame exceeds
  8.2 ms; streaming the 8 km terrain decodes 473 tiles at 16 MiB and 202
  at 4 MiB, the streamer spends at most 0.12 ms a frame, and handing eight
  tiles costs about 1 ms (`terrain/fly/frame-times.md`, `NOTES.md`).
- **Edits are world state** (9024a12, 6a8b6ee): every og.terrain.edit
  joins an edit log in the world's og.terrain block; snapshots keep it,
  the world hash covers it and a restore replays it.
- **Scatter** (1ced372): og-render places scattered copies once per
  32 x 32-cell chunk, caches their instances and draws each chunk with one
  instanced draw, fading copies by a dither at full size in every pass.
  The 36,000-tuft grass costs +0.46 ms (was +4.1 to +4.5 ms;
  `costs-p2-after-scatter.md`, `terrain-grass-before-after.png`).
- **Asset streaming groups** (8f19c0b, 8ae1f46, 1317314, a3d3870,
  79e8dc7, 98db532, 5481b09): a scene's `stream` blocks (distance, volume
  or request-only triggers, priority) are cooked into one pack per group
  with a proxy for each grouped glTF; the runtime admits wanted groups by
  priority within a memory budget (256 MiB by default) and swaps real
  meshes, textures and sounds in behind the stand-ins' handles, so the
  world hash never depends on what is resident; IDL module `stream`
  (og.stream.status, request, release, budget, setBudget). `og build --publish`
  ships the first group as files the shell preloads and each other group
  as its pack, fetched on demand and cached by the service worker for
  offline play. `packages/engine/tests/asset-stream.test.ts` flies a
  three-group path natively and on the web, online and offline
  (`groups/`).
- **Shared render change**: 8ae1f46 adds a replaced-texture pass to
  og-render's sync step (`RenderScene::set_texture`,
  `set_texture_upload`), outside the terrain, scatter and loading paths.
- **Gate**: `OG_GATE_JOBS=2 scripts/check.sh --all` at 58c2466,
  2026-10-19, Apple M4 Pro shared with other agents' work: **PASSED, 9 of
  9 steps in 134 s**: fmt 1 s, clippy 2 s, wasm-build 7 s, test 85 s (470
  Rust tests, 5 skipped), wasm-opt 76 s, size-gate 48 s, drift 1 s,
  bun-test 38 s, determinism 1 s.
- **Wasm**: publish wasm 3,064,943 B brotli, 73.1 % of the 4 MiB budget
  (+86,482 B over the terrain round's 2,978,461 B): the tile pyramid and
  its streamer, the scatter chunks, the streaming groups and the
  replaced-texture pass.
- **Shore foam on vulkan-cpu**: traced to the sea's single 24 km
  triangle, which llvmpipe interpolates metres off once the near plane
  clips it; any tessellation brings the foam and the lake back
  (`foam/NOTES.md`, 58c2466). Fixed in the culling round, which draws large
  water in cells of at most 256 m (ec54c60).

## Status 2026-10-19, shell own-window round

A top-level Stage build with the platform's Stage overlay in its own
document now has that overlay as its Player: when the page is not framed,
carries no `window.__OG_GAME` and was told its own origin as the Stage host
(`window.__OG_STAGE_HOST__ === location.origin`, the og-probe wire's own
test, now one helper, `shell/wire.ts` `stageInPage`), the shell's Player
bridge (`shell/player.ts`) speaks Player protocol 2 to its own window by
`postMessage`, hears only its own window at its own origin, and ignores its
own echo (a `hello` naming a `substrate` or listing no `capabilities`). The
overlay's answers count as the Player's: its `hello` with `started: true`
starts play with no gate of the shell's, og.platform's store and saves go
to it, and nothing of og.platform's reaches the Stage origin's
`localStorage` (the page's memory answers what the overlay does not); no
service worker is registered there. Framed pages and `__OG_GAME` pages are
unchanged, and any other top-level page still plays alone on the browser's
storage. A `result`'s `time` now passes through the bridge from every
transport (framed, HTTP, own window), so `og.platform.info().clock` reads
`player` as soon as a Player stamps its answers. Details:
`docs/web-export.md` §Player protocol 2; what the overlay must do to answer
is ask 10 in `docs/platform-asks.md`.

Proof: `tests/platform-stage.test.ts` (6 tests against a stand-in overlay
answering in the same window), `tests/web/player.ts` in Chrome (65 checks,
22 of them new: the Stage build with an in-document stand-in overlay at
desktop and phone sizes, and a silent overlay that never answers). Each
fails when its code is broken: without the echo filter, the `time`
pass-through, the source and origin check, the `self` peer, the `__OG_GAME`
precedence or the page's memory store. Transcripts, screenshots and
receipts: `tests/evidence/P2/shell-self/`.

## Status 2026-10-19, phase 2, tooling round

Branch `p2-tooling`; it closes owed rows 22 and 34 and records row 35's
stop (the table below says which commits). Evidence:
`tests/evidence/P2/tooling/` (`web-gfx-webgpu.json`, `web-gfx-webgl2.json`,
`m4-m5-tooling.md`, `m4-gltf-reload.json`, `m5-sweep-8-frames.json`,
`wasm-build-relink.md`, `break-and-restore.md`, before and after frames).

- **og-tests' gfx scenes in the browser** (22): `crates/og-tests-web`
  renders every scene of og-tests on an OffscreenCanvas in headless
  Chrome, on WebGPU and on WebGL2, and `tests/web/reference-gfx.ts`
  compares each with its tier's reference or metal's under og-tests' RMSE
  rule. All 47 applicable scenes pass on each tier (31 s and 52 s on the
  Apple M4 Pro, ANGLE Metal). WebGL2 has references of its own for `msaa`
  (only that tier renders it), `showcase` and `overlay` (GPU particles on
  the CPU path) and `water-3d` (cubemap reflection); `taa` does not render
  there. The web-reference step runs both tiers.
- **Browser frame times** (22): the seven perf scenes per tier beside
  metal's in `crates/og-tests/perf/<tier>-chrome-apple-m4-pro/`; WebGPU
  against WebGL2, ms per frame: showcase 2.15 / 2.58, spheres-20k 6.49 /
  6.89, sprites-20k 1.11 / 1.23, gltf-reference 1.38 / 1.64 (taken at load
  averages 50 to 191).
- **M4, glTF edit to reload** (34): through `og dev` on the scaffolded
  template, eight edits of the dropship's hull colour reload in 90 to
  113 ms (og dev's figure; 145 to 251 ms from the write). The first run met
  the time but not the change: the reload kept the models already loaded.
  Fixed in og-runtime (changed cooked glTF loads again into its asset
  index) and guarded by `packages/engine/tests/reload-asset.test.ts`.
- **M5, timed sweep** (34): `og probe sweep` over the trickster's
  `Spin.speed`, 8 values of 30 ticks, gives 8 distinct 1280×720 frames in
  298 to 559 ms round trip (target 1 s).
- **wasm-build relink**: the publish variant builds in its own target dir,
  so a rerun with nothing changed costs 0.4 and 0.5 s instead of 16.7 and
  12.2 s (cargo 1.99 keeps one fingerprint per unit whatever its features).
- **wgpu and naga 31** (35): not on crates.io; the newest is 30.0.1
  (2026-08-22), which the engine already uses. The row stays open.

Gate: `OG_GATE_JOBS=2 scripts/check.sh --all` on dbf3ae1 (this commit adds
documents only), with other agents' builds on the machine (load 18 to 48):
**PASSED, 9 of 9 steps in 115 s**: fmt 2 s, clippy 0 s, wasm-build 1 s,
test 67 s, wasm-opt 62 s, size-gate 45 s, drift 1 s, bun-test 20 s,
determinism 25 s. The run before it failed bun-test once: `i18n.test.ts`'s
`beforeAll` cook passed bun's 5 s hook timeout at load 40 and passed on the
rerun. `scripts/check.sh web-reference`'s new steps pass on both tiers
(above). The publish wasm is **3,029,921 B brotli, 72.2 %** of the budget
(the render round's merge recorded 3,028,729 B, +1,192 B: the model
refresh).

Left out: the wgpu 31 upgrade (unpublished); `og dev` killed by a signal
from outside leaves its bun child running (a clean exit and Ctrl-C stop
it; the reload test kills both); og-tests' `og-gfx` capture binary stays
native (it reads scenes from disk; the browser harness replaces it there);
browser frame times were taken with other agents' gates on the machine, so
they are upper bounds.

## Status 2026-10-19, player-page round

The engine's page is the play page itself (design §4): `og build` writes
`<meta name="og-player" content="frame page">` and og-build.json's
`player { protocol: 2, transports: [frame, page] }` into every build, and a
top-level page the platform published (`window.__OG_GAME`) speaks Player
protocol 2 to its play origin over same-origin HTTP (`shell/transport.ts`:
`/__player/v2/<game>/<release>/{hello,call,report}`, the hand-off code from
the fragment redeemed in the hello's body, the Player's credential keys and
outbox, session reports from the person's start), as origingame
`docs/player-protocol.md` §The top-level engine page specifies. Framed pages
keep the `postMessage` path unchanged, so a release built now plays under
today's Player too. At the top level the shell draws what the Player page
drew: the cover's backdrop behind the boot splash and Back (with its own
gate, `100dvh`, the safe areas, one audio unlock, fullscreen on the gesture
and the PWA hint, which it already had). The og-probe wire also answers the
page's own window when told its own origin, for the Stage overlay the
platform injects into a top-level Stage build. Details:
`docs/web-export.md` §The top-level page; the platform's side is ask 9 in
`docs/platform-asks.md`.

Proof: `tests/platform-page.test.ts` (6 tests against a stand-in play
origin), `tests/web/player.ts` (37 checks in Chrome: the framed Player, then
the top-level page at desktop and phone sizes), `tests/web/wire.ts` (125,
with the in-page host; its Tune checks moved off `Tuning.gravity`, which
the runner dropped in E7), `tests/web/export.ts` (40). Transcripts,
screenshots and receipts: `tests/evidence/P2/player-page/`.

What the platform's pin round must do: move
`agent-server/scripts/e2b/origin-engine.json` to this round's merge,
rebuild the layout and remount the knowledge (`og-platform-sdk` changed),
and have the platform's player-page work live first: the
`/__player/v2/` routes on game hosts, the game-host ingress admitting `POST`
with `Authorization` there, Play's 307 to a `page` release's entry, and the
Stage page's in-document mode. Until then a release built from this shell
is framed exactly as before.

Owed to that pin round from this side: the platform's Stage page now
answers a framed build's Player protocol as a stand-in Player (play
started, saves in its own memory), but a top-level Stage build has no
parent, so this shell plays it alone, draws its own gate and keeps
`og.platform` in the Stage origin's browser storage, which every Game's
builds share. Before the pin moves, the shell must also speak protocol 2
to its own window when `window.__OG_STAGE_HOST__` is its own origin (as
the og-probe wire already does), and the Stage overlay answer it there.

## Status 2026-10-19, Phase 2, core round

Branch `p2-core`, 11d6820..71dabed. It closes the engine defects and gaps
the archetypes round (below) found. Each public addition went through the
IDL in its own commit, with its skill text and the generated wiki.

- **og.Rng from reads, loud undeclared access** (d2bd748, 71dabed): a fork
  reads the seed and the label and moves nothing, so a system with og.Rng
  in `reads` forks the same streams as one that writes it, where it used to
  answer NaN. The reproduction is `packages/engine/tests/fixtures/rng-readonly`.
  An undeclared resource or event read or write throws in og run and
  og test, naming the system and the record. A publish build refuses it
  and logs it once per system and access through the probe's diagnostics.
  og-core states both rules.
- **The host's clock** (fc97734): `og.platform.now`, a `time` on every
  og.platform answer, and `og.platform.info(world).clock` (`player`,
  `browser`, `system`, `fixed` or `none`). These are host input between ticks, so a replay
  records them. Natively the clock is the system's; `og run --clock` and
  `og test` fix it. On the web the Worker uses the browser's clock until
  the Player's hello and answers carry `time` (`docs/platform-asks.md`,
  open ask 9). An answer that already carries a `time` is taken as is.
- **A kept store** (cd1f0d3): an `og test` spec with `store: "keep"` runs
  each `then` step as a new process over one local platform store, on a
  host clock the spec moves. `relaunch:<seconds>` stays for one-process
  tests.
- **Input `at`** (56fc770): key and pointer events carry when they happened
  within the tick that reads them (-1 to 1 of a tick). The time is handed
  in with the event, so the world stays deterministic on both hosts.
- **og.audio.outputLatency** (094df16): the time from world time to the
  speaker. On the web it is the AudioContext's output and base latency
  plus the ring's prime. Natively it is the device's latency (cpal's playback
  instant less its callback instant) and the mixer's buffer, and headless it
  is 0.
- **AudioSource.startOffset** (f3fb31f): a clip can start part-way, to the
  sample (og-audio's `mix.rs` asserts the PCM).
- **Archetypes** (d2bd748, 4868361, b6d889a): their workarounds are gone.
  - The roguelike's Foes and Loot declare og.Rng in their reads.
  - The idle game stamps its save with og.platform.now and credits the
    time away from the store answer's `time`. `tests/away.og.ts`
    relaunches two hours later in a second process.
  - The rhythm game judges a press at its `at`, against what was heard
    (the output latency, plus the player's own offset), and resumes from
    the beat it paused on (`tests/timing.og.ts`, `tests/pause.og.ts`).
  - Each README's Known limits keeps only what is still open.
- **Playtests**: every template's `og test` passes.

  | Template | Suites |
  |---|---|
  | game | 2 |
  | ui-kit | 25 |
  | fx-showcase | 10 |
  | idle | 10 |
  | match3 | 6 |
  | rhythm | 11 |
  | roguelike | 9 |
  | runner | 8 |
  | topdown | 7 |
  | tower-defense | 9 |

- **og polish** (`tests/evidence/P2/core/polish/`):

  | Archetype | Taste | Feel |
  |---|---|---|
  | idle | 97 | 74 |
  | roguelike | 99 | 80 |
  | rhythm | 99 | 74 |

  Taste is unchanged from the archetypes round, and every feel row but
  frame time reads 92 to 100. The frame-time row is 0 because these runs
  used the debug og at load 28. The archetypes round's runs used a release
  og. Run og polish on the rhythm game from its own directory: its tests'
  chart lookup falls back to the working directory.
- **Gate**: `OG_GATE_JOBS=2 scripts/check.sh --all` at 71dabed,
  2026-10-19, at load 16–29, **PASSED, 9 of 9 steps in 147 s**. The steps
  took fmt 1 s, clippy 1 s, wasm-build 9 s, test 105 s, wasm-opt 104 s,
  size-gate 31 s, drift 1 s, bun-test 25 s and determinism 2 s. Builds were
  warm from the path-aware run before it.

  Two earlier runs did not count. The first `--all` stopped at clippy on
  a `manual_contains` lint in the undeclared-access check, which 71dabed
  fixes. The path-aware run before this gate timed out e5's every-feature
  hash test at 5 s under twelve parallel files. That test passes alone in
  0.66 s and in this gate.

  Path-aware runs after the gate, with only this section and the evidence
  changed, hit two timing bounds at load 28–36. e5's hash test timed out
  again. og-audio's `one_second_of_a_busy_mix_renders_in_bounded_time` took
  1,621.9 ms against its 1,000 ms debug bound. Both pass when run alone,
  and both tests predate this round. The round's mixer change touches only
  a voice that has not started yet.
- **Wasm**: the publish wasm is 2,982,720 B brotli, 71 % of the budget,
  +4,259 B over 2,978,461 B.
- **Merge of main** (44df5d7: the animation and physics round and the
  render round): only the generated files (bindings.json and the wiki,
  regenerated by `og api gen`) and this document conflicted. api.ogt,
  shell.ts and worker.ts merged cleanly, keeping both the input `at` and
  the compressed-texture Worker. `OG_GATE_JOBS=2 scripts/check.sh --all`
  on the merge, 2026-10-19, at load 38–194: **PASSED, 9 of 9 steps in
  423 s**. The steps took fmt 3 s, clippy 1 s, wasm-build 27 s, test 58 s,
  wasm-opt 69 s, size-gate 53 s, drift 2 s, bun-test 126 s and determinism
  416 s. Before it, the path-aware run at load 148 failed the audio-mix
  timing bound again (1,606.1 ms). The publish wasm is 3,033,764 B brotli,
  72 %, +5,035 B over the render round's merge (3,028,729 B).
- **Merge of main** (7438978: the UI round): again only the generated
  bindings and wiki (regenerated) and this document conflicted.
  `OG_GATE_JOBS=2 scripts/check.sh --all` on the merge **PASSED, 9 of 9
  steps in 246 s**. The steps took fmt 3 s, clippy 1 s, wasm-build 25 s,
  test 166 s, wasm-opt 150 s, size-gate 69 s, drift 2 s, bun-test 41 s and
  determinism 14 s. The publish wasm is 3,061,522 B brotli, 72 %.

  The archetypes' `og test` fails on main at 7438978 alone (built from
  `git archive`), with the same failures as on this merge, so it is not the
  merge:
  - Every `frames` suite misses its Metal references (the UI round's
    restyle, for example tower-defense's title at RMSE 0.033).
  - Tower-defense's title takes no tap or pad press before about 0.35 s,
    so `lose`, `pad`, `pause` and `replay`, which press Play at 0.3 to
    0.4 s, never start a round. A tap at 0.4 s starts one.
- **Evidence**: `tests/evidence/P2/core/break-and-restore.md` breaks every
  new check once.
- **Not done**:
  - The TS types do not restrict `world.resource` to the declared set.
    IDL callbacks carry no generics, so that needs an og-idl generator
    change; the runtime check is the guard meanwhile.
  - A gamepad press carries no `at`, because the shell polls pads once a
    display frame.
  - The native device latency is not tested: the gate has no audio
    device and builds without `native-audio`. The web value is tested.
  - The Player's own clock waits on ask 9.

## Status 2026-10-19, Phase 2, UI round

Branch `p2-ui` from main at 2dae5fa (`phase1-e7` and the gate-speed
work). It closes owed rows 15–19, 21, 33, 41, 42, 45 and 46 (the table
below names each commit) and nothing else:

- **Rich text** (row 33, 6139efe): `og.ui.richText` lays out markup from
  the string table: `[b]`, `[color=]`, `[size=]`, `[font=]`, `[icon=]`,
  `[link=]` (a child element keys, pads and touch reach) and per-glyph
  `[wave]`, `[shake]`, `[pulse]`; shaped and right-to-left pieces in visual
  order. The kit's confirm and alert messages carry it in five locales.
- **Accessibility tree** (row 33, 38694c6): each element's role, name,
  value and states in `dump_ui` and `og.ui.a11y()`; focus order is the
  tree's.
- **Glass** (row 33, 42d7294): og-render blurs the frame under overlay
  panels through a six-level pyramid; skin line `glass <part> <blur> <fill>
  [tint]`; the scifi pack's dialogs and panels are glass (+0.83 ms a frame
  for four large panels, `tests/evidence/P2/ui/glass-cost.md`).
- **Scrolling both ways** (row 33, 79efd08): `ScrollDesc.axes`, the handoff
  to the outer scroller, a fling, a draggable scrollbar.
- **Text**: per-glyph tracking from the ramp and a distance-field glow
  (17, 18, 0df11b1); the kit's small type at 9 px (42, f9bad51).
- **Input and feel**: UI sounds in their input's tick (15, 59ce0b4),
  spring hover and press (16, 814fdad), focus moves heard (41, ca81fb8),
  pads 1–3 with focus of their own (21, ca179c0), the shop's refusal
  shaken and heard (45, d00f20c), the kit's hits felt (ffc0eb3), runner's
  held press heard (46, 54b59df).
- **`og skin pack`** (19, f4c236b): nine-patch PNGs to a skin's atlas and
  lines.

Row 43's engine side (the platform's danger tokens in og-ui's tokens and
a button contrast test) is cf932f8 and 5d982db from `p2-anim-physics`;
this round does not repeat it, and the merge of main keeps both.

### Gate

`OG_GATE_JOBS=2 scripts/check.sh --all` on f4c236b (this commit adds
documents and polish evidence only), 2026-10-19, Apple M4 Pro with four
other rounds building on the machine (load average 30): 169 s, fmt 2 s,
clippy 2 s, wasm-build 14 s, test 107 s, wasm-opt 108 s, drift 2 s,
determinism 2 s; bun-test failed on one test,
`tests/e5.test.ts`'s every-feature hash at 5,265 ms against bun's 5,000 ms
timeout under twelve parallel workers (the file takes 756 ms alone), and
cancelled size-gate. `--only bun-test size-gate` then passed both (19 s;
72 engine tests in 13.8 s). `og test templates/ui-kit` passes 29 suites and
runner 8; references this round re-recorded say why in their commits, with
before and after images in `tests/evidence/P2/ui/rerecord/`, and
`tests/evidence/P2/ui/break-and-restore.md` has how each new test was broken
once.

### Wasm

**2,974,147 B brotli, 70.9 % of the 4,194,304 B budget**, from 2,943,180 B
(70.2 %): +30,967 B for rich text, the accessibility tree, glass (its
shaders and pyramid), two-axis scrolling, extra pads, springs and the glow.

After merging main (44df5d7: terrain and water, the archetypes, the
animation and physics round with og-ui's danger tokens, the render round),
19 kit references and og-ui's sample-screen, changed on both sides, are
re-recorded with this round's text and main's danger ink together (0.04 to
0.50 % outliers against main's, 3.2 and 6.6 % for the scifi sheets' glass;
RMSE 0.006 to 0.045; before and after in
`tests/evidence/P2/ui/rerecord/merge-main-*.png`), and the IDL's outputs
regenerated. `OG_GATE_JOBS=2 scripts/check.sh` **PASSED, 9 of 9 in 965 s**
at load 40–70, then `--all` **PASSED, 9 of 9 steps in 111 s**: fmt 2 s,
clippy 1 s, wasm-build 10 s, test 82 s, wasm-opt 74 s, size-gate 25 s,
drift 2 s, bun-test 18 s, determinism 8 s. `og test` passes ui-kit (29
suites) and runner (8, now `templates/archetypes/runner`). The publish wasm
is **3,057,601 B brotli, 72.9 %** for all the rounds together.

### Polish

`og polish` before (E7) and after (`tests/evidence/P2/ui/polish/`), on
metal with other rounds' builds on the machine:

| Template | Taste | Feel | Findings |
|---|---|---|---|
| ui-kit | 90 → 91 | 62 → 70 | hit feedback 0 → 100, sound on input 38 → 73, UI sounds 62 → 65, text size → 98; frame time 0 under load (E7 read it 0 too); contrast 84 is row 43's default primary and danger labels, until `p2-anim-physics` merges |
| runner | n/a | 99 → 95 | 0 (row 46's flow input closed; sound on input 100); frame time 80 under load, not a regression in the game |

### Left out

- A shell mirroring `og.ui.a11y()` into the DOM for screen readers: the
  tree is there to mirror; the web shell's mirror is the Player round's.
- Glass over other overlay content (it blurs the scene only) and bidi
  reordering inside one rich-text span (pieces are reordered, not runs
  within a piece).
- The touch HUD's action buttons sound on the sfx bus, not the ui bus, so
  ui-kit's UI sounds stays at 65; the sounds-off playtest is silent by
  intent.
- Packs whose fonts have small x-heights still measure 7–8 px on some
  labels; a pack's ramp line for base raises them.

### Fix on main (`p2-ui-fix`, from b2532cd)

Four archetypes failed `og test` on main after the round:
- idle, rhythm, roguelike and tower-defense in `frames`;
- tower-defense in lose, pad, pause and replay, whose title refused a tap on
  Play before 0.367 s.

The cause was og-ui's stagger. It kept each staggered control inert until its
own place in the cascade had played. The round's springs rest the hover at
exactly zero, so skinned buttons joined the cascade for the first time.

Now the cascade is only seen. A control takes input once the panel's own
entrance, still inert, has played, and the title takes a tap from 0.267 s as
before the round. New test: `skin.rs`
`a_staggered_panels_controls_take_presses_once_the_panel_has_arrived`, broken
twice (break-and-restore.md).

Nine archetype frames were re-recorded for this round's text: the tokens'
tracking on display titles and headings, and the distance-field glow of idle's
and rhythm's `textGlow`. Each reason and the three-build attribution are in
`tests/evidence/P2/ui/archetype-frames.md`, with before and after images in
`rerecord/archetype-*.png`.

`og test` passes:
- the seven archetypes: idle 10, rhythm 11, roguelike 9, tower-defense 9,
  match3 6, runner 8 and topdown 7 suites;
- fx-showcase 18, game 2 and ui-kit 29 suites.

`OG_GATE_JOBS=2 scripts/check.sh` **PASSED, 9 of 9 in 1059 s** at load 30–50.
web-reference was not run, because no og-ui reference changed. The publish
wasm is 3,061,786 B brotli, 72 %.

### Fix on main (`p2-ui-webgl`, from 7a5e320)

The tooling agent's web-reference run found two failures on main.

**Every WebGL2 renderer panicked, from 42d7294 (the overlay glass).** wgpu's GL
backend translates each shader to GLSL ES 3.0 when it creates a pipeline.
`fs_glass` asked its pyramid for its level count with `textureNumLevels`, which
GLSL ES 3.0 lacks (`Features(TEXTURE_LEVELS)`), so the overlay pipeline could
not be created. fx-showcase on WebGL2 reported "engine did not start:
unreachable", and every graphics scene failed there. The design's §8 reduced
tier has no texture queries.

Now the glass pass hands the shader the level count. The pyramid's own sprite
params, `[0, 0, 0, LEVELS]`, are bound in place of the zero params (`glass.rs`
`pyramid_params`), and `fs_glass` reads `sprite_params.msdf.w`. The level of
detail is the same number as before, so the WebGPU and native images are
unchanged. New test: `crates/og-render/tests/webgl2.rs` runs wgpu's own naga
translation (WGSL to GLSL ES 300, WebGL flags) over every entry point of
`overlay.wgsl` and `glass.wgsl`, so a texture query or other GLSL ES 3.0 gap
fails natively without a browser. Broken once (break-and-restore.md).

**fx-showcase on WebGPU failed every station at b2532cd, and still did on
7a5e320.** Its start threw `models/trickster.glb#tex0: not transcoded`, so the
scene was never built (post 99.5 % outliers, RMSE 0.75). It is not the UI round
or the texture streaming round. Since d2bd748 (undeclared access is loud), the
TypeScript `Engine` constructor learns the build from the probe's `hello`, and
every probe call ran a tick boundary. With `deferStart`, the tick boundary is
where GameDesc.start runs. So start ran inside `createEngine`, before the Worker
attached its renderer and transcoded the bundle's Basis textures, and `og run`
with a renderer started before its offscreen surface was attached. Now `hello`,
which only says who is speaking, runs no tick boundary (`engine.rs`
`answer_call`). New test: `packages/engine/tests/runtime.test.ts`, with
deferStart, start waits for the first tick boundary and `hello` is not one.
Written before the fix and failing on it (break-and-restore.md).

`scripts/check.sh --only web-reference` **PASSED in 65 s**:
- fx-showcase 5 of 5 stations on WebGPU and 5 of 5 on WebGL2;
- E5 17 of 17 on each tier.

No reference was re-recorded.

`OG_GATE_JOBS=2 scripts/check.sh` passed fmt, clippy, wasm-build, test, wasm-opt,
size-gate and drift. bun-test timed out once, at load 140, in web-export's
`the same inputs give the same bytes`; that test passes alone in 0.2 s. A
rerun, `scripts/check.sh --only bun-test`, **PASSED** with the engine package's
108 tests. The publish wasm is 3,151,777 B brotli, 75 %.

## Status 2026-10-19, phase 2, og-ui tiers round

Branch `p2-ui-tiers` from 44d12a8 (evidence: `tests/evidence/P2/ui-tiers/`,
`NOTES.md` there indexes it). Owed row 20 is done: every og-ui reference
frame now exists on all four tiers.

- **vulkan-cpu** (2131b15): all 74 frames (og-ui's four scenes, 22 desktop
  and 21 phone kit screens, 15 locale screens, 12 skin sheets), recorded
  with `OG_UI_CAPTURE=1` on lavapipe in the og-lavapipe image with
  zigbuild Linux binaries. The scenes move into
  `crates/og-ui/tests/support/scenes.rs` so the browser can build them too.
- **Browser, og-ui's scenes** (de92f08): og-tests-web includes that file
  and draws `og-ui/<name>`; `reference-gfx.ts` judges them against
  `crates/og-ui/references/<tier>/`. Both tiers match metal exactly except
  widget-set (RMSE 0.00046, as the native metal run).
- **Browser, the kit** (8f1d3af): reference.ts takes `--size`,
  `--references` and `--sheet`; `tests/web/reference-ui.ts` runs ui-kit's
  16 reference playtests through it (the full set, 70 frames; nothing left
  out, a tier takes 100 to 140 s) and `scripts/check.sh web-reference`
  runs it on both tiers.
- **Against metal** every frame is within the rule on every tier. Eleven
  frames have outliers, the same on vulkan-cpu, webgpu and webgl2, at most
  0.038 % (RMSE 0.0089, `kit-desktop-settings`): the large titles' edges
  and the gallery's scrollbar thumb, a fraction of a pixel of coverage.
  The three non-metal tiers agree there with no outliers between them, so
  the difference is metal's. Every other frame is within RMSE 0.0040.
- **Break and restore**: a mirrored vulkan-cpu sample-screen fails og-ui's
  gfx test on lavapipe; a wrong webgl2 locale frame, a wrong webgpu skin
  sheet and a mirrored webgpu glass fail the browser runners; restored,
  all pass.

### Gate

`OG_GATE_JOBS=2 scripts/check.sh` on 8f1d3af (the docs and evidence of
the next commit were uncommitted and touch no check): fmt, clippy (75 s),
wasm-build, wasm-opt (212 s), size-gate (3,176,490 B brotli, 75 % of
4,194,304; no crate in the wasm build changed this round), drift (186 s)
and test (722 s) passed. bun-test failed after 506 s on five timeouts
and no assertion, while a lavapipe container ran beside it (104 pass):
render.test.ts "trails, beams and ribbons" and "material and sprite
effects" at 5 s and web-export.test.ts "the same inputs give the same
bytes" at 120 s (the known load flakes), plus asset-stream.test.ts
"what is resident never changes the simulation" at 5 s and "og build
--publish ships each group as its pack" at 120 s (not on the known list;
this round changes nothing they run). This stopped determinism. Neither the `--only bun-test`
rerun nor determinism nor `scripts/check.sh --only web-reference` ran:
the coordinator now runs the gate on main after the merge. The browser
kit and scene frames were checked directly instead: `reference-ui.ts`
and `reference-gfx.ts` without `--record` reproduce every frame on
webgpu and webgl2 at RMSE 0.
On lavapipe, gfx 5 of 5 and kit_gfx 1 of 1 pass at RMSE 0 against
`vulkan-cpu`; `og test templates/ui-kit` without capture passed 17 of 29
suites (frames-desktop, frames-phone, locales, skin-cartoon,
skin-cartoon-phone, skin-fantasy-phone and the eleven playtests without
frames) before it was stopped for time. skin-fantasy and the neon, paper,
pixel and scifi skin suites (nine), plus sounds, touch and tutorial,
passed only in the recording run, so their comparison against the
recorded `vulkan-cpu` sheets is unverified.

## Status 2026-10-19, phase 2, 3D SPH round

Branch `p2-sph` (evidence: `tests/evidence/P2/sph/`, `NOTES.md` there
indexes it). Owed row 29 is done.

- **The simulation** (6dcdc98, 87ea4bd): `og.Fluid3D`, a 3D particle fluid
  by double density relaxation (Clavet et al. 2005), the 2D fluid's model,
  rather than position-based fluids: one neighbour search and two passes a
  substep instead of PBF's solver iterations, stable at 60 Hz with two
  substeps, and groups (four fluids at most, layering by `density`,
  blending by `mixing`), emitters (`emitRate` up to `maxCount`) and sink
  boxes carry over. Neighbours come from a spatial hash in a fixed order;
  particles collide with the 3D colliders and the terrain's height field
  and stay in optional `bounds`. All of it is og.physics state (snapshots,
  restore, the world hash). CPU per 60 Hz tick, native and wasm32 alike:
  about 0.9 ms for 2,000 particles at one substep, 1.8 ms at the default
  two, 4 ms for 4,000 and 10 ms for 8,000, about double where they pile
  deep.
- **Determinism** (a513e20): the beyond-rigid-bodies scenario pours two 3D
  fluids of one group into a terrain bowl through an emitter and a sink;
  host debug and release and wasm32 scalar and simd128 agree (physics_hash
  8301574762329360262).
- **The public API** (71257f0): api.ogt's physics module declares
  `og.Fluid3D` and `og.physics.fluid3dParticles`, regenerated with `og api
  gen` into the bindings, schema, d.ts, ABI, glue and wiki.
- **The surface** (200b6fe, 7fc64e7, 2f9e908, 792aa2b): og-render draws
  every visible 3D fluid as one screen-space surface on all four tiers (no
  storage buffer, no compute): sphere imposters into a depth and a
  thickness target, a separable bilateral smoothing, and a composite in
  the main pass that refracts and absorbs the scene behind, lights the
  body with the sun, its cascades, the image-based light and the clustered
  local lights, reflects the environment and writes its depth. About
  1.1 ms of GPU on the M4 Pro for a fluid covering a third of a 720p frame,
  whatever the particle count. Marching cubes was neither built nor
  measured; the case against it (a density grid and a mesh rebuilt every
  frame, on the CPU for WebGL2) is reasoning only. Shared render-graph
  changes: `post.rs` gains `FramePass::Fluid3d` and its five transients
  (`fluid.depth`, `fluid.zbuf`, `fluid.thick`, `fluid.blur`,
  `fluid.smooth`), read by the main pass; `lib.rs` forces the depth
  prepass and the split main pass while a fluid has particles.
- **fx-showcase** station 23, fluids3d (332ec84): a blue pool and a heavy
  orange fluid poured into it by an emitter, sinking under it, with a
  playtest of 12 checks and references on metal, vulkan-cpu (1974892),
  webgpu and webgl2 (829143e), all within the RMSE rule against metal.
- **Docs** (7e63835): the og-game skill and the wiki teach og.Fluid3D and
  its CPU and GPU cost; og-physics' and og-render's READMEs gain their
  phase 2 sections.

### Gate and wasm

`OG_GATE_JOBS=2 scripts/check.sh --all` on 8658977 (this section and
the NOTES' last edit uncommitted, documentation only), 2026-10-19, with
other agents' gates on the machine (load average 28 to 96): **PASSED, 9 of
9 steps in 412 s**: fmt 1 s, clippy 0 s, drift 2 s, wasm-build 27 s,
wasm-opt 123 s, test 125 s, size-gate 47 s, bun-test 27 s, determinism
407 s (every build agrees, physics_hash 8301574762329360262). The run
before it (fmt 4 s, clippy 65 s, wasm-build 23 s, wasm-opt 98 s, size-gate
64 s, drift 32 s, test 360 s) failed at bun-test at load 96 on two timeouts
in code this round does not touch: i18n.test.ts's beforeAll (`og cook` of
ui-kit, 5 s) and web-export.test.ts (120 s); alone they pass in 4 s, and
the rerun passed. The publish wasm is **3,052,777 B brotli, 72.8 % of the
4,194,304 B budget**, 24,048 B (0.57 points) more than the 3,028,729 B
before this round. Earlier, `scripts/check.sh web-reference` passed in
168 s (18 of 18 stations on webgpu and webgl2, fluids3d included).

## Status 2026-10-19, phase 2, render round

Branch `p2-render`; it closes owed rows 23, 28, 30, 31 and 32 (the table
below says which commits). Evidence: `tests/evidence/P2/render/`
(`forward-plus.md`, `materials-and-hooks.md`, `compressed-textures.md`,
`break-and-restore.md`, before and after frames).

- **Forward+** (30): point and spot lights are clustered on the CPU into 16
  x 9 tiles by 24 logarithmic slices (10-40 us for 256 lights), storage
  buffers on the full tier and 16 KiB uniform tables on WebGL2; up to
  `MAX_LIGHTS` 256 in view, 64 per cluster, the brightest at the camera kept
  past either cap. Frame time, 8-light loop against clustered: metal
  `lights-256` 0.24 ms (8 lights drawn) against 0.58 ms (243 drawn; drawing
  them all per pixel costs 5.8 ms), `point-spot-lights` 0.42-0.66 against
  0.46-0.56 ms; Chrome's lights station 1.0-1.2 against 1.7-2.0 ms on
  WebGPU and 1.8-1.9 against 2.2-2.4 ms on WebGL2, its fx station unchanged.
  The phase 1 lighting line, the one not met as written, is met.
- **The moon** (28): a disc in the procedural sky with its phase, size,
  intensity, colour and direction (`og.Sky`, `og.DayNight`).
- **Materials** (31): normals by the cofactor matrix under non-uniform
  scale, vertex tangents in glTF's frame (file, cook or generated; normal
  maps had read inverted), clearcoat, and the `materials` gfx scene.
- **Material shaders** (23): on skinned meshes, and in the shadow passes and
  the depth prepass; `og.render.setMaterialShader` for a loaded model's
  materials.
- **Compressed textures** (32): cooked textures reach the GPU as BC7/BC5,
  ASTC 4x4 or ETC2/EAC with their mips (the template's: 2.5 MB instead of
  10.1 MB, load time unchanged); normal maps now cook with y in alpha.
- **vulkan-cpu**: the probe, `og.render.capabilities`, `og polish` and the
  reference directories name a CPU Vulkan adapter (lavapipe, llvmpipe,
  SwiftShader) `vulkan-cpu`, a GPU one `vulkan`.

Gate: `OG_GATE_JOBS=2 scripts/check.sh --all` on 872b002 (this commit adds
documents only), with other agents' builds on the machine (load 75):
**PASSED, 9 of 9 steps in 173 s**: fmt 5 s, clippy 14 s, wasm-build 37 s,
test 107 s, wasm-opt 102 s, size-gate 27 s, drift 9 s, bun-test 23 s,
determinism 3 s. `scripts/check.sh web-reference` (Chrome for Testing 150)
matches fx-showcase's five frames and twelve stations on WebGPU and WebGL2;
`og test` passes fx-showcase (13 suites) on metal and on lavapipe and the
game template (2). The publish wasm is **2,958,860 B brotli, 70.5 %** of the
budget (2,943,180 B before the round, +15,680 B).

After merging main (9c9d6d3: terrain and water, the archetypes, animation
and physics), terrain's lit shader and the water reflection pass use the
clusters too (the reflection builds its own over the main view's kept
lights), and the render stations moved to `src/render.ts` as 19-22 at x
2100-2300; the moon frame was re-recorded on every tier for the move
(`fx-showcase-moon-relocated-before-after.png`). `OG_GATE_JOBS=2
scripts/check.sh --all` on the merge **PASSED, 9 of 9 steps in 291 s**:
fmt 4 s, clippy 2 s, wasm-build 19 s, test 225 s, wasm-opt 202 s,
size-gate 42 s, drift 10 s, bun-test 35 s, determinism 28 s.
`scripts/check.sh web-reference` matches all 17 stations on WebGPU and
WebGL2; `og test` passes fx-showcase (18 suites) on metal and lavapipe.
The publish wasm is **3,028,729 B brotli, 72.2 %** (main recorded
3,013,367 B, +15,362 B).

Left out: clearcoat maps (WebGL2's skinned lit pipeline has no texture slot
left), a compute cluster build (the CPU build costs tens of microseconds),
upload timings in the browser (the Worker's transcoding is timed, the GPU
upload natively), and ASTC and ETC2 frames off Apple silicon (lavapipe and
desktop Chrome take BC).

## Phase 2, terrain and water round (2026-10-19)

Branch `p2-terrain-water`, 0ee78f1..HEAD on 2dae5fa. Closes owed row 24.

- **Terrain** (0ee78f1, 290f4a4, 9a9d6da, 5be9281): a `type terrain`
  .ogt (a 16-bit grey PNG or an EXR heightmap, an RGBA control map, up to
  four layers with slope and altitude rules) cooked into zlib'd 16-bit
  tiles of 128 samples, deterministically; drawn by og-render as a
  geometry clipmap (nested `ring` × `ring` levels snapped to twice their
  spacing, the finer level's hole cut from one index buffer, CDLOD's morph
  at the edges: no cracks at six camera positions, fewer levels from afar),
  normals built on the GPU, layers planar or triplanar weighted by control
  map × slope × altitude and sharpened by their heights, casting and
  receiving cascaded and local shadows under the fog; a rapier height
  field (SHAPE_HEIGHTFIELD, split along the renderer's diagonal) that the
  character controller walks on; a navmesh `.ogt`'s `terrain` entry bakes
  its walkable slopes. IDL module `terrain`: og.Terrain, og.terrain.load,
  sample, bounds, edit (raise, lower, flatten, smooth, paint, seen by the
  renderer, physics and sample), layer, setLayer; og-game §11 and
  `wiki/terrain.md`.
- **Water** (row 24; 0ee78f1, 5be9281): og.Water's planarReflection (the
  camera mirrored in the surface, an oblique near plane at the water,
  0.25 to 1 of the resolution, cheaper than the screen-space march it
  replaces), shadows (0 to 1), caustics on what lies below, and the
  underwater view (the water's fog, Snell's window from below).
- **Scatter** (b35a73b, fd4456d): og.TerrainScatter grows instanced copies
  of a mesh over a layer near the camera, fixed to a grid on the terrain,
  shrinking away with distance; og-tests' terrain-grass scene.
- **fx-showcase** (15ff230): the `terrain` station, a 256 m valley with a
  path painted in the control map, a lake with a planar reflection of the
  mountain, and a character walking up a knoll, on all four tiers.
- **References**: og-tests terrain-near, -mid, -far, -grass,
  water-planar, -shadows, -underwater on metal and vulkan-cpu (reproduced
  at RMSE 0; every earlier vulkan-cpu reference still passes); the
  fx-showcase station on metal, vulkan-cpu, webgpu and webgl2.
- **Costs and frame times** (`tests/evidence/P2/terrain-water/`): terrain
  +0.1 to +0.55 ms on the M4 Pro, its shadow passes +0.2 to +0.35 ms,
  water shadows and caustics within noise, the underwater view +0.4 ms,
  the 36,000-tuft grass +4.1 to +4.5 ms, all CPU submit. Every scene draws
  in under 2.5 ms on metal (grass 4.3 ms); the terrain station's median
  frame is 1.34 ms on Chrome WebGPU and 2.43 ms on WebGL2, under the
  8.3 ms desktop budget.
- **Gate**: `OG_GATE_JOBS=2 scripts/check.sh --all` at ad06e6e,
  2026-10-19, Apple M4 Pro shared with four other agents' work:
  **PASSED, 9 of 9 steps in 559 s** (9 min 19 s): fmt 3 s, clippy 14 s,
  wasm-build 184 s, test 219 s, wasm-opt 200 s, size-gate 61 s, drift 6 s,
  bun-test 150 s, determinism 2 s.
- **Wasm**: publish wasm 2,978,461 B brotli, 71.0 % of the budget
  (+35,281 B over 2,943,180). The terrain's load-time decoding lives in
  og-cook and miniz_oxide, both at opt-level z in `[profile.wasm]`.
- **Not done** at the round's end, since closed by the phase 3 streaming
  round below unless noted: terrains were decoded whole at load (now
  streamed tile by tile, bc5c5ea); a terrain lay along world x and z
  (now turned and scaled with its entity, 9941ffd, 6b9221b); edits were
  asset changes outside snapshots (now world state, 9024a12); scattered
  copies shrank instead of fading and were rebuilt on the CPU every frame
  (now dithered and placed once per cached chunk, 1ced372); lavapipe draws
  no shore foam on `terrain-far`'s 24 km single-quad sea (fixed by
  ec54c60, which draws large water in cells of at most 256 m:
  `tests/evidence/P3/streaming/foam/NOTES.md`).

## Status 2026-10-19, Phase 2, archetypes round

The four Discover archetypes still owed are built on the engine (branch
`p2-archetypes`): `templates/archetypes/tower-defense` (Path Guard),
`roguelike` (Lamp Dive), `idle` (Spark Orb) and `rhythm` (Three-Lane Beat).
Each is a complete small game with a title, pause, settings (volumes,
language, reduced motion; the rhythm's offset), an end card, keyboard,
gamepad and touch, safe areas and a phone layout, an og-ui HUD on a
recoloured style pack, en and zh-CN tables, synthesised sounds (CC0, by
its `tools/make-assets.py`, byte-for-byte deterministic, as is the cook),
Metal reference frames and a README naming the skill sections it
exercises. The create CLI scaffolds all seven archetypes
(`--archetype`), and the knowledge skills (og-ai, og-audio, og-feel,
og-game, og-i18n, og-platform-sdk) point at them.

| Archetype | Playtests (`og test`) | `og polish` taste / feel | Publish archive (zip) | The game's own content |
|---|---|---|---|---|
| tower-defense | 9 of 9 | 99 / 100 | 5,738,666 B | 1,386,864 B |
| roguelike | 9 of 9 | 99 / 100 | 5,620,824 B | 1,295,752 B |
| idle | 9 of 9 | 97 / 94 | 6,151,017 B | 1,803,759 B |
| rhythm | 10 of 10 | 99 / 99 | 6,039,807 B | 1,822,905 B |

The archive is `og build . --publish`'s `dist/` zipped, most of it the
engine's wasm and web runtime; the game's own content (`cooked/` and
`game.js`) is under the Discover prototypes' 2 MB asset budget in each.
The polish reports (`tests/evidence/P2/archetypes/<name>/`) were made with
og and the native addon built release from this checkout: on this machine
(load 15 to 70 from other agents' work) frame time's p95 follows the load,
not the games, whose median step and render are 1–3 ms
(`tests/evidence/P2/archetypes/frame-time.md`, with a per-tick series).
`tests/evidence/P2/archetypes/break-and-restore.md` breaks one check of
each.

Gate: `OG_GATE_JOBS=2 scripts/check.sh --all` on 3c5c7d4 with this
commit's documents and skill pointers, 2026-10-19, load 11–35: **PASSED,
9 of 9 steps in 99 s** (fmt 1 s, clippy 0 s, wasm-build 6 s, test 59 s,
wasm-opt 62 s, size-gate 28 s, drift 1 s, bun-test 22 s, determinism
1 s; the publish wasm 2,943,180 B brotli, 70 % of the budget). The four
archetypes' `og test` runs (9, 9, 9 and 10 suites) pass beside it.

Engine gaps the games work around, each in its README's Known limits:
og.ui's `settingsScreen`, `resultsScreen` and `idleHud` carry English
words and number formats of their own, so the settings, end cards and the
idle HUD are built from widgets; `og polish` reads og.ui's phone title
menu (a bottom sheet) as HUD. Closed by the Phase 2 core round (above):
og.platform's answers carry the host's time and og.platform.now reads it,
so the idle game credits the time away from the save's stamp (a second
`og test` process over a kept store is its test; `relaunch:<seconds>`
stays for tests in one process); og.audio.outputLatency and the input
events' `at` let the rhythm game judge a press when it happened and as it
was heard (the taps' calibration stays, as the player's own offset); an
AudioSource's `startOffset` starts a clip part-way to the sample, so a
resume carries on from the beat it paused on. The two engine defects met
on the way are fixed there too: a system with `og.Rng` in `reads` only
forks the same streams as one that writes it (the roguelike's Foes and
Loot declare it in their reads), and an undeclared resource or event access throws in og run
and og test and is refused and logged once per system in a publish build.

## Status 2026-10-19, phase 2, animation and physics round

Branch `p2-anim-physics` (evidence: `tests/evidence/P2/anim-physics/`,
`NOTES.md` there indexes it). Owed rows 25, 26 and 27 are done; 29 (3D SPH)
was not started and is left for the next round.

- **Animation retargeting** (aaea84c, 707b6b2): og-anim maps a humanoid's
  22 bones from its names (Mixamo, VRM, Unity, Rigify, Unreal, Blender
  detected; a `type rig` .ogt overrides the scheme or any bone) and
  retargets a clip onto another rig: rotations moved in model space with
  each bone's rest corrected, the hips' travel and height scaled by the
  legs' ratio (root motion kept), planted feet held on the ground by
  two-bone leg IK, lowering the hips where a leg cannot reach. The cook
  writes an animation library's clips into a model retargeted
  (`animation { rig library footIk }` in its import settings, re-cooked
  when the rig or the library changes); `og.anim.retarget`,
  `og.anim.humanoid` and `og.load.rig` do it at run time, cached. Feet stay
  planted with zero height error and slide on rigs at 1.267 and 0.689 times
  the library's legs (11.5 mm off with forward kinematics alone).
- **2D ragdolls** (row 27; bf15ab1, 53331df): og.Ragdoll in the 2D profile,
  rapier2d capsules on hinges about z limited by `jointLimit`, following
  the animation at weight 0 and blending as in 3D; `og.Physics.profile` can
  now be set at run time.
- **Cloth self-collision** (row 26; bf15ab1): `og.Cloth.thickness` (a
  sorted spatial hash, pairs pushed apart every pass, a point moving at
  most half the thickness a step), `substeps` and `friction`. It costs 5 to
  6 times the plain cloth: about 0.5 ms a tick for a 20 x 20 sheet, 2.4 ms
  at 32 x 32.
- **Fluids** (row 25; bf15ab1, 53fa607): up to sixteen metaball fluids a
  frame, packed four to a field; `og.Fluid2D.group`, `density` and
  `mixing`: fluids of one group (four at most) step as one particle set,
  heavier ones sinking, held apart at their boundary unless they mix, their
  mix evening out where they touch and drawn as one surface whose colour
  follows it. A group costs one fluid of all its particles plus 7 to 14 %.
- **fx-showcase** gains four stations, retarget, ragdoll2d, cloth and
  fluids (4a716f7), each with a playtest and references on metal,
  vulkan-cpu, webgpu and webgl2 (f7a0d88, f270817).
- **Determinism**: `tools/wasm-determinism` (the gate's determinism step)
  now hashes og-physics' beyond-rigid-bodies scenario with the new cloth,
  fluid group and 2D ragdoll in 3D and 2D, and og-anim's walk with its
  retargeting; host debug and release and wasm32 scalar and simd128 agree
  (ca02ca3). f37a586 restores the E5 joints station's 3D ragdoll bit for
  bit after the 2D ragdoll refactor.
- **og-ui** (cf932f8, done here at the coordinator's request though it is
  og-ui's): the platform's new danger tokens, the danger button on the
  danger tint, a contrast test of every button role (row 43), and the og-ui
  and ui-kit references the colour touched, re-recorded.

### Gate and wasm

`OG_GATE_JOBS=2 scripts/check.sh --all` on 5d982db, 2026-10-19, with other
agents' builds on the machine: **PASSED, 9 of 9 steps in 743 s**: fmt 5 s,
clippy 21 s, drift 5 s, determinism 46 s, wasm-build 235 s, wasm-opt
305 s, test 347 s, size-gate 92 s, bun-test 154 s. The publish wasm is
**2,975,644 B brotli, 70.9 % of the 4,194,304 B budget**, 32,464 B
(0.8 points) more than the E7 round's 2,943,180 B.
After merging main (11d6820, the terrain and water round and the
archetypes) in 00238f6, `OG_GATE_JOBS=2 scripts/check.sh --all` on 00238f6
**PASSED, 9 of 9 steps in 123 s** (incremental after the path-aware run),
with the publish wasm at **3,013,367 B brotli, 71.8 %** for both rounds
together.

## Status 2026-10-19, E7 round

Phase 1 is closed by the E7 round (tag `phase1-e7`). Against
`docs/design.md` §11 phase 1 it delivers the ECS with reflection, the fixed
deterministic tick and snapshots, `.ogt`/`.ogb` and the cook (glTF,
textures, scenes, fonts, sound, tilemaps, navmeshes), PBR with shadows and
IBL, input, camera, audio, the command channel and the probe (protocol 5
plus the wire), hot reload, `og`, the IDL pipeline (d.ts, wiki, schema, the
hosts' glue), the project template and three archetype games (runner,
match3, topdown) where two were asked, all proved by gfx references on
metal, vulkan-cpu, WebGPU and WebGL2 and by playtests. It also brought
forward much of phase 2: skeletal animation with blend trees and IK, rapier
in 2D and 3D with character, platformer and vehicle controllers, joints,
ropes, cloth, 2D fluids and ragdolls, navmeshes and og.ai, particles, the
post stack, decals, atmosphere and water, material shaders, the og-ui
toolkit on the platform's tokens with MSDF text and six style packs, Tune,
i18n, og.platform over the Player protocol and `og polish`. Lighting is
the one phase 1 line not met as written: forward lighting is an 8-light
uniform loop, not Forward+ (row 30). Phase 2 starts with terrain (a
clipmap), Forward+ clustered lighting, animation retargeting, and the L
items of the owed table: planar water (24), 3D SPH (29), Forward+ (30) and
og-ui's rich text, accessibility tree, glass and two-axis scrolling (33).

### Gate

`scripts/check.sh --all` on db65e5a (this commit adds documents only),
2026-10-19, Apple M4 Pro with another agent's platform validation on the
machine: **PASSED, 9 of 9 steps in 237 s**: fmt 1 s, clippy 4 s,
wasm-build 44 s, test 42 s, wasm-opt 37 s, size-gate 21 s, drift 4 s,
bun-test 83 s, determinism 1 s. Beside it: `og test` passes on all six
templates (game 2 suites, ui-kit 25, fx-showcase 9, runner 8, match3 6,
topdown 7; 160 s in all), and `make web-reference` (Chrome for Testing
150) matches all 26 browser frames, fx-showcase's five and eight E5
stations on WebGPU and WebGL2 (27 s). Every committed reference frame
passes; the ones this round re-recorded say why with before and after
images (ba1042a kinsoku, c6d5793 the ragdoll, 93d86e1 the lazy stations,
706bee2 the other tiers). `tests/evidence/E7/break-and-restore.md` has how
each change's test was broken once.

### Wasm

The publish wasm (`og_host_web-publish`, wasm-opt -Oz, brotli -q 11):
**2,943,180 B brotli, 70.2 % of the 4,194,304 B budget** (3,021,619 B,
72.0 %, before the round's `[profile.wasm]`, fcca471: the load-time crates
at opt-level z; the per-tick crates at z cost a quarter of a frame or half
the physics step again) (`tests/evidence/E7/wasm-size.md`: every experiment, its
speed, and what going below 65 % takes).

### Polish

`og polish` on the six templates (`tests/evidence/E7/polish/`; taste n/a
where the HUD is screen-space text rather than og.ui):

| Template | Taste | Feel | Findings | Closed this round |
|---|---|---|---|---|
| game | 100 | 87 | 1 (idle frames: the guard playtest stages its scenario without pressing Start) | — |
| ui-kit | 90 (73 before) | 62 | 111 (rows 41–43, 45; the rest read a catalogue as a game) | polish read dialogs, sheets and the cutscene as HUDs, dimmed text under scrims and touch controls (da05a4d) |
| fx-showcase | 100 | 97 | 0 | the E5 stations built lazily (93d86e1) |
| runner | n/a | 99 | 0 (row 46 below the threshold) | — |
| match3 | n/a | 100 (88) | 0 | the dealing press heard (c49780a) |
| topdown | n/a | 97 (86) | 0 | a hit taken holds the world; footsteps heard (62f52f3) |


### Owed

After E7: 14 rows done (1–13 and the platform asks, 37), row 14 partly (the
wasm at 70.2 %, not under 65 %), and open 25 for the engine's phase 2
(15–35, 41, 42, 45, 46), 3 for the platform (36, 38, 43) and 3 for the
owner (39, 40, 44). After the phase 2 UI round (P2 UI below): 11 more done
(15–19, 21, 33, 41, 42, 45, 46); row 43's engine side came with the
animation and physics round (cf932f8).

After the phase 2 animation and physics round: rows 25, 26, 27 and 43 done
too; 29 (3D SPH) still open.

After the phase 2 3D SPH round: row 29 done too.

After the phase 2 showcase round: three new rows open (47–49), row 12 now
covers the archetypes' typecheck too.

Every item a round's status left owed or out, in one place; the round
sections below point here instead of repeating them. Size: S a day or
less, M a few days, L a workstream. Owner: **engine** is this repository
(E7 when this round closes it, else phase 2), **platform** is
`origingame/origingame` (read only from here), **owner** is a decision or
resource only the owner can give.

| # | Item | Round | Size | Owner | State |
|---|---|---|---|---|---|
| 1 | `og.ui.shake` for game code (only a refused press shakes today) | E3 | S | engine, E7 | done, da47e5e |
| 2 | Nine-slice edges and centre tiled, not stretched, when the skin says so | E3 | S | engine, E7 | done, 4a19260 |
| 3 | Phone references for the six ui-kit skin packs (desktop only) | E3 | S | engine, E7 | done, d7c95b3 |
| 4 | Camera zones that scroll between rooms instead of only clamping; topdown's door uses it | E4 | S | engine, E7 | done, 0c630ce |
| 5 | Core commands that read and set resources; game commands' usage text in `help` | E4 | S | engine, E7 | done, cea6fab |
| 6 | `og.tilemap.getTile` on a terrain layer answers the resolved tile, not the terrain bit | E4 | S | engine, E7 | done, 0006005, e7f4966 |
| 7 | Ragdoll bodies follow the animation at weight 0 (the runtime never called `set_ragdoll_targets`) | E5 | S | engine, E7 | done, c6d5793 |
| 8 | runner on the 2D platformer controller, topdown on the 2D character movement | E5 | S | engine, E7 | done, 142b9da, 497deee |
| 9 | Japanese line breaking with kinsoku rules in og-ui (CJK breaks between any two characters) | E6 | S | engine, E7 | done, ba1042a |
| 10 | `og polish`'s text measures count icon names and disabled buttons | E6 | S | engine, E7 | done, 891b4b6 |
| 11 | fx-showcase's E5 stations built lazily per station (its playtests and polish frame time dropped to 100/88) | E6 | S | engine, E7 | done, 93d86e1 |
| 12 | `templates/fx-showcase/src/main.ts`'s type errors; templates are not typechecked in the gate | E6 | S | engine, E7 | done, c5295be; the archetypes too, through `tsconfig.templates.json`'s `templates/archetypes/*/src` glob, third-person (Ember Vale) among them |
| 13 | Native and Chrome world hashes of the three archetypes compared (match3 differed, not investigated) | E6 | S | engine, E7 | done, 85c2bd0 |
| 14 | The publish wasm under 65 % of the 4 MB brotli budget without removing features (71 % before E7) | E7 | M | engine, E7 | partly: 72.0 % to 70.2 % (fcca471); under 65 % is row 44 |
| 15 | UI sounds start one tick after the input (`og.ui.sound` runs after `og.audio`) | E3 | S | engine, P2 UI | done, 59ce0b4 |
| 16 | Springs for the skin's hover and press (eased curves today) | E3 | S | engine, P2 UI | done, 814fdad |
| 17 | Per-glyph tracking from a skin's ramp (needs per-glyph spacing in og-render's text) | E3 | M | engine, P2 UI | done, 0df11b1 |
| 18 | A real text glow (an MSDF outline in og-render; four offset copies today) | E3 | M | engine, P2 UI | done, 0df11b1 |
| 19 | An atlas tool for skins drawn by hand, beyond `make-skins.py` | E3 | M | engine, P2 UI | done, f4c236b |
| 20 | `vulkan-cpu` and browser references for og-ui's scenes, the UI kit, the skin packs and the locale screens (metal only) | E2, E3, E6 | M | engine, phase 2 | done, 2131b15, de92f08, 8f1d3af (all 74 frames on vulkan-cpu, webgpu and webgl2; the browser kit runner in web-reference) |
| 21 | Pads beyond the first in og-ui | E2 | M | engine, P2 UI | done, ca179c0 |
| 22 | Browser runs of og-tests' own gfx scenes (a wasm harness) and GL-tier references (`msaa` among them) | M3, E2 | M | engine, phase 2 | done, 0ac7c34, c43947e, 9470fc3, eaf0517 |
| 23 | Shader hooks on skinned meshes; shadows and the depth prepass displaced by `vertex_offset` | E5 | M | engine, phase 2 | done, 12e93b5, 49dfd0d (`og.render.setMaterialShader`), aa81258 |
| 24 | Planar water reflection, shadowed water, an underwater view | E5 | L | engine, phase 2 | done, 0ee78f1 (og-render), 5be9281 (og.Water fields), 15ff230 (fx-showcase terrain station) |
| 25 | More than four metaball fluids a frame, fluids that interact | E5 | M | engine, phase 2 | done, bf15ab1, 53fa607, 4a716f7 (sixteen a frame, groups that layer by density and blend by mixing) |
| 26 | Cloth self-collision | E5 | M | engine, phase 2 | done, bf15ab1, 4a716f7 |
| 27 | 2D ragdolls | E5 | M | engine, phase 2 | done, bf15ab1, 53331df, 4a716f7 |
| 28 | A moon disc in the night sky | E5 | S | engine, phase 2 | done, 1df9797, 9037875, a5cdb89 |
| 29 | 3D SPH (3D water is the height field) | E5 | L | engine, phase 2 | done, 6dcdc98, a513e20, 71257f0, 200b6fe, 2f9e908, 792aa2b, 332ec84 (double density relaxation in 3D, a screen-space surface on all four tiers, the fluids3d station) |
| 30 | Forward+ clustered lights beyond the 8-light uniform loop (CPU tile table on WebGL2) | M3, M6 | L | engine, phase 2 | done, a061113, 1296a02, f381a66 |
| 31 | A normal matrix for non-uniform scale, vertex tangents, clearcoat, and a gfx scene of the M6 material features | M6 | M | engine, phase 2 | done, 12e93b5, 49dfd0d, eb869b8, aa81258 |
| 32 | The runtime loaders transcode to the tier's compressed texture format and upload it (RGBA8 today) | M4, M6 | M | engine, phase 2 | done, 028c2fc |
| 33 | og-ui's rich text, accessibility tree, glass, two-axis scrolling (`crates/og-ui/README.md` §What phase 2 owes) | M6 | L | engine, P2 UI | done, 79efd08, 6139efe, 38694c6, 42d7294 |
| 34 | The M4 "glTF edit to reload in 300 ms" run and the M5 timed 8-frame `sweep`, filed as evidence (`og dev` prints the first) | M4, M5 | S | engine, phase 2 | done, 3d7ad18, 5119a09 |
| 35 | wgpu and naga 31 when published, kept only if every reference frame passes unchanged | Toolchain | S | engine, phase 2 | open: not published (2026-10-19 the newest on crates.io is 30.0.1, of 2026-08-22) |
| 36 | `og polish` measures frame time on the tier it runs on (metal here), not on the executors' lavapipe | E6 | S | platform | open |
| 37 | The eight platform asks (`docs/platform-asks.md`) | E6 | M | platform | done: landed on the platform 2026-10-19, the engine speaks it (96f4ebd) |
| 38 | The platform side of the cutover: merge `origin-engine-cutover`, pin this revision | E1 | S | platform | done: the platform merged the cutover (origingame `7e5789d89`) and pins `cf3cb0b` (`agent-server/scripts/e2b/origin-engine.json`), with the knowledge mount, local layout, runner tests and workspace stack checked on it; the next pin bump moves to this main |
| 39 | A third native tier beside `metal` and `vulkan-cpu` (the browser tiers stand in) | E4 | M | owner | open |
| 40 | Mobile measurements: the iPhone first frame and frame times (Safari, WebGPU), the mid-range Android view benchmark | M6, E1 | S | owner (devices) | open |
| 41 | og-ui plays no sound when keyboard or gamepad focus moves (ui-kit's sound on input 38 in og polish) | E7 | S | engine, P2 UI | done, ca81fb8 |
| 42 | The kit's small type: the shop cards' item names (`caption`, 7 CSS px of ink), the idle HUD's generator names and the `meta` units, under og polish's 9 px | E7 | S | engine, P2 UI | done, f9bad51 |
| 43 | The default tokens' primary and danger button labels at 3.2:1 and 2.9:1 against their fill (og polish wants 4.5:1; WCAG's large text 3:1) | E7 | S | platform (the design tokens) | done: the platform's tokens of b71ae8fac, og-ui synced with the danger tint in cf932f8 (every role 4.5:1 or more, `crates/og-ui/tests/button_contrast.rs`) |
| 44 | A publish wasm per profile (measured 63.3 % 3D, 59.5 % 2D at opt-level 3): two shipping variants, `og build --publish` choosing by the game's profile, and the size budget read per download instead of as the sum of every wasm the gate builds | E7 | M | owner (the budget's meaning), then engine | open |
| 45 | `og.ui.shake` shown in a template (ui-kit's shop refusing a purchase) | E7 | S | engine, P2 UI | done, d00f20c |
| 46 | runner: a press during the game-over card's hold is silent (og polish: 1 of 3 inputs in `flow`) | E7 | S | engine, P2 UI | done, 54b59df |
| 47 | vulkan-cpu references for the third-person archetype (metal, webgpu and webgl2 recorded) | P2 showcase | S | engine, phase 2 | open |
| 48 | The fantasy pack's labels under 4.5:1 (Set out 3.36, footer 3.5, Resume 3.84, Again 3.99, 完成 4.41, 语言 4.43 in Ember Vale's polish) and third-person's HUD Jump and Kindle without a ui-bus sound | P2 showcase | S | engine, phase 2 | open |
| 49 | The IDL's og.IkChain docs: a `"world"` target is in the skeleton body's Transform parent's frame, not true world | P2 showcase | S | engine, phase 2 | open |

## 0. Rules that shape the plan

- **The IDL is the contract between workstreams.** `crates/og-api/api.ogt`
  is written first and every crate implements against it. A workstream that
  needs an API change edits the IDL in its own commit; generated outputs are
  CI-checked for drift.
- **Evidence per milestone, not per sprint.** Each milestone names the
  scripted run, the gfx reference frames or the conformance suite that proves
  it. A milestone without its evidence is not done.
- **Four parallel workstreams, one integrator.** Core, Render, Assets, Tools
  run as separate agent threads on their own crates; the integrator merges
  weekly, owns the IDL and the templates, and runs the full suite.
- **2D is in phase 1.** Five of the six Discover archetypes are 2D; the
  profile (orthographic camera, sprite batch, 2D physics via rapier2d) costs
  about two weeks on top of the 3D core and makes the first games real.
- **Both backends from day one.** WebGPU and WebGL2 are the same code with a
  tier flag; the spike has already paid for the differences. Native Metal
  (developers) and Vulkan (executors) come for free from wgpu.

## 1. Workstreams and crates

| Workstream | Crates | Owns |
|---|---|---|
| **Core** | `og-core`, `og-ecs`, `og-idl`, `og-api`, `og-runtime` | ids, handles, reflection macro and schema, `.ogt`/`.ogb`, world, archetypes, queries, schedule, snapshot/restore, deterministic rng and math, the IDL and its generators |
| **Render** | `og-render`, `shaders/` | render graph, materials, instancing, lights, CSM, IBL, sky, mips, MSAA/TAA, bloom/tonemap, 2D sprite and text batches, capture readback |
| **Assets** | `og-cook`, `templates/` | glTF import (meshopt, LODs, tangents), KTX2/Basis, audio (QOA), MSDF fonts, scene compile, cook stamps, the project template and archetypes |
| **Tools** | `og-probe`, `og-cli`, `og-host-web`, `og-host-node`, `packages/engine`, `packages/create`, `packages/knowledge` | command channel, probe, dumps, `og` (build/run/cook/probe/test/doctor), the two hosts, the npm packages, wiki generation |
| **Integrator** | workspace, CI, `tests/`, `docs/` | IDL stewardship, weekly merges, gfx/perf suites, templates in sync, the milestone evidence |

## 2. Milestones

| # | Weeks | Deliverable | Evidence |
|---|---|---|---|
| M0 Scaffold | 1 | workspace with every crate stubbed, CI (fmt, clippy, test, wasm build, size gate, generated-file drift), `og` skeleton, `tests/` harness with gfx reference compare, the spike renderer moved into `og-render` behind the new crate boundary | CI green on an empty game; the spike frame reproduced from `og-render` byte-identical to `.spike-out/native.png`. **Filed** (`tests/evidence/M0/`, 2026-10-17): `spike-frame-reproduced.png` (byte-identical to the spike). CI green: the gate run at 49ed935 is `tests/evidence/M6/check-summary.md`. |
| M1 World | 1–3 | `og-core`: `#[derive(Reflect)]` → schema JSON, `.ogt` reader/writer with round-trip, handles, rng, math; `og-ecs`: archetype storage, queries, system schedule, resources, events, snapshot/restore | logic tests: 10k-entity structural churn, snapshot→restore→identical world hash, `.ogt` round-trip on every template file; a Rust-only "bouncing boxes" world advanced 3600 ticks twice gives the same hash. **Filed** (`tests/evidence/M1/`, 2026-10-17): `cross-target-determinism.md` (bouncing boxes 3600 ticks, the same hash on host and wasm32). The churn, snapshot and `.ogt` round-trip tests run in the gate (`cargo test -p og-ecs`, `og-core`) with no file of their own. |
| M2 API | 2–4 | `og-api`: `api.ogt` v1 (world, components, systems, resources, transform, camera, input, time, rng, load, probe), generators for Rust traits, wasm-bindgen glue, napi glue, `.d.ts`, JSON schema, wiki pages; typed views over columns with tick-stamped invalidation; `og-host-web` Worker + OffscreenCanvas; `og-host-node` | a TypeScript hello-world (spawn 10k entities, one `Move` system) runs identically on both hosts: same world hash after 600 ticks; the spike's view benchmark re-run through the real API within 2× of the raw loop. **Filed** (`tests/evidence/M2/`, 2026-10-17): `hello-world-hash-node.json`, `hello-world-hash-web.json`, `hello-world-two-hosts.md`, `view-benchmark.json`, `wasm-size.json`. |
| M3 Renderer | 3–6 | render graph; material system from reflected material assets; automatic instancing by (mesh, material); directional + point/spot lights; 3-cascade CSM; IBL + sky pass; mip generation; MSAA (GL) / TAA (WebGPU); bloom, tonemap, exposure; 2D sprite batch and MSDF text; capture readback on both hosts | gfx reference frames for 12 scenes (committed PNGs, per backend tier); the spike's glTF scene within ±10 % of the spike's frame time; 20k moving sprites ≤ 2 ms on the M4 Pro. **Filed** (`tests/evidence/M3/`, 2026-10-17): `showcase-metal.png`, `showcase-vulkan-cpu.png`, `perf-baselines.json`, `feature-costs.md`. 11 Metal and 11 vulkan-cpu references are committed (`crates/og-tests/references/`); the browser tiers run og-tests' scenes since the phase 2 tooling round, with WebGL2 references for `msaa`, `showcase`, `overlay` and `water-3d` (`tests/evidence/P2/tooling/web-gfx-*.json`). |
| M4 Cook | 5–8 | `og cook`: glTF → `.ogb` (meshopt vertex/index codec, cache ordering, LOD chain, tangents), textures → KTX2 Basis (UASTC for normals, ETC1S for colour), audio → QOA, fonts → MSDF atlas, scenes `.ogt` → `.ogb`; incremental by hash; cook stamps checked at load; runtime loaders; hot reload by snapshot + bundle swap | cook of the template and archetypes is deterministic (same bytes twice); a glTF edit → `og cook` → reload shows the change in ≤ 300 ms with world state kept; cooked size of the template ≤ 60 % of its sources. **Filed** (`tests/evidence/M4/`, 2026-10-17): `cook-determinism.json`, `cook-report-template.md` (cooked 35.7 % of sources), `cooked-gltf-reference.png`, `cooked-gltf-reference-compare.json`, `texture-psnr.md`. The glTF edit → reload run is filed in `tests/evidence/P2/tooling/m4-m5-tooling.md` (8 reloads, worst 112.8 ms in `og dev`, 251 ms from the write; the edit shows since 3d7ad18). |
| M5 Probe | 6–9 | `og-probe`: command channel (core actions of design §3.3), module-registered commands, probe protocol 5 members + `capture`, `sweep`, `reload`, `snapshot`, `restore`, `dump`, `commands`; native loopback WebSocket; in-page request ring; `publish` trim | the platform's probe conformance suite (`origin/packages/contracts`) green on both hosts; `og run --commands "0.5:touch_move_to:Start;0.6:touch_start;0.7:touch_end;2.0:screenshot;2.2:dump_world;2.5:quit"` on the template produces the frame and the dump; a `sweep` over a tunable yields 8 frames in ≤ 1 s native. **Filed** (`tests/evidence/M5/`, 2026-10-17): `probe-conformance-test-output.md`, `probe-conformance-transcript.json`, `scripted-run-dump-world.json`. The timed 8-frame `sweep` is filed in `tests/evidence/P2/tooling/m4-m5-tooling.md` (298 to 559 ms); the `og run --commands` run on the template is M6's `tools-template-*`. |
| M6 Games | 8–11 | input (pointer, keys, gamepad, touch), camera controllers, audio (AudioWorklet ring, cpal), 2D physics (rapier2d) and 3D rigid bodies (rapier3d, static + dynamic, no character controller yet), minimal UI (panels, buttons, text, stable ids), project template, `npm create @origingame/game`, two archetypes rebuilt (`runner` 2D, `match3`), wiki generated, skills and scaffold in `packages/knowledge` | both archetypes playable in a browser and through `og run --commands`; their playtest scripts pass headless; the knowledge package validates against the platform's `packages/game-knowledge` schema; first-frame on an iPhone and the Android view benchmark recorded. **Filed so far** (`tests/evidence/M6/`, 2026-10-17 UTC; partial, M6 is not done): Core `core-cross-target-determinism.md`, `core-input-edges-transcript.json`, `core-break-and-restore.md`; Render `render-showcase-metal.png`, `render-showcase-vulkan-cpu.png`, `render-shadow-map-256.png`, `render-shadow-map-2048.png`, `render-texture-formats.md`, `render-test-run.md`, `render-feature-costs.md`, `render-perf.md`; UI `ui-sample-screen-metal.png`, `ui-dump-ui-sample.json`, `ui-test-run.md`; Audio `audio-determinism.json`, `audio-mix-timing-release.json`, `audio-mix-timing-debug.json`, `audio-scene-waveform.png`, `audio-test-run.md`; ECS `ecs-state-blocks.md`; Physics `physics-pile-determinism.json`, `physics-pile-determinism-debug.json`, `physics-wasm-size.json`, `physics-test-run.md`; Tools `tools-template-run-frame.png`, `tools-template-dump-world.json`, `tools-template-dump-entity.json`, `tools-template-run.md`; gate `check-summary.md`, `size-gate.json`. Not yet: the archetypes, their playtests, the knowledge package against the live platform schema, the mobile numbers. |
| M7 Platform | 11–13 | `@origingame/engine` published (wasm + native addon per platform via optionalDependencies, `.d.ts`); E2B `engine` class image (bun + engine + lavapipe); runner `og.rs` for `stage_capture`/`stage_record`/`level_report`; mode table without substrate; one Discover prototype produced end to end by a thread on the new engine | the thread's session transcript with frames; `engine` image ≤ 600 MB; `bun deploy/ovh/state.ts` shows the new class live; the platform goal's phase-1 row filled in |

Dependencies: M1 → M2 → everything TypeScript-facing; M0 → M3 (renderer
work starts from the spike immediately); M4 needs M1's `.ogb`; M5 needs M2's
hosts; M6 needs M3, M4, M5; M7 needs M6.

### Status 2026-10-19, E6 round

- **Navigation and AI** (bf15ac2..8e631ee): og-nav (grid A* and jump point
  search without corner cutting, flow fields, a tiled navmesh baked by
  `og cook` from `type navmesh` .ogt with obstacles that rebuild only their
  tiles, steering, the `og.nav` agent system; 200 agents a tick in 0.25 ms
  on a 256 grid) and og-ai (behaviour trees and state machines as `type
  behaviour` .ogt, checks on blackboard keys and component and resource
  fields, sight cones against a physics ray and hearing, the game's
  commands; 200 guards think in 0.28 ms a tick over og.nav's 0.24), IDL
  modules `nav` and `ai`, `dump_ai` and `dump_nav`. The topdown slime runs
  on a tree; templates/game has a 3D guard on a state machine and a baked
  navmesh.
- **Audio** (802c95b..a3ce21b): per-bus inserts (high- and low-pass,
  compressor, ducking, a reverb send), per-voice low-pass for distance and
  occlusion, voice priority and a cap, sample-accurate `startAt`, fades,
  the `og.Music` clock with `og.MusicBeat`; `listen` (probe member and
  command) and `og run --audio`; 18 procedural presets synthesised by
  `og cook` (`og sfx`). 32 voices through every insert mix in 1 % of real
  time; the busy scene renders the same PCM twice.
- **Platform services** (917a861, 88764c5): `og.platform` (store with
  revisions, leaderboards, telemetry, Origin Point, AI marks, settings
  helpers) answered at a later tick; a local JSON store natively, Player
  protocol 2 on the web with a localStorage fallback; `dump_platform`.
  `docs/platform-asks.md` records eight asks.
- **Localization** (7ae351a..ee3b32e): og-i18n (string tables, ICU
  messages, CLDR plurals, negotiation, direction), rustybuzz shaping and
  right-to-left layout in og-ui, per-locale font chains, the cook's key
  check, ui-kit in en, zh-CN, ja, ko and ar against metal references
  (`tests/evidence/E6/i18n/locale-sheet.png`).
- **og polish** (da276d8): playtests played headless and scored against
  og-taste and og-feel (contrast, text size, HUD, transitions, UI sounds,
  latency, hit feedback, idle ticks, frame time, sound on input, locale
  keys), reports with frames and fixes mapped to skill sections. Its first
  run found runner and match3 silent (their sounds were cooked, never
  played), ui-kit's UI sounds off by default and templates/game's Start
  press silent; all four fixed (a98019f..1fcb224). Reports for the six
  templates are in `tests/evidence/E6/polish/`.
- **Knowledge**: skills og-ai, og-audio, og-i18n and og-platform-sdk;
  og-taste §11 and og-feel §14 for the polish measures; og-game's map and
  performance budget.
- Full gate at b6fa67a in 4 min 5 s; publish wasm 2,862,533 bytes brotli
  (68 % of the budget, +433,997 over E3's 2,428,536), shaping (rustybuzz, ttf-parser)
  the largest new share (`tests/evidence/E6/wasm-size.json`). Evidence
  `tests/evidence/E6/`.
- Topdown's `vulkan-cpu` references for the slime on og.ai were recorded
  after the merge into main, in the og-lavapipe container with the x86_64
  Linux build (reproduced at RMSE 0; within RMSE 0.0031 of `metal`).
- Owed: §Owed rows 9–13, 20 and 36–37.

### Status 2026-10-19, E5 round

- **Atmosphere, water, material shaders and physics beyond rigid bodies**
  (db38b41..HEAD, branch `e5`):
  - **og-physics** gains joints (fixed, hinge and slider with limits and
    motors, spring, rope, ball) and the 3D kinematic character controller
    (slopes, steps, ground snapping, moving platforms, pushing).
  - It gains the 2D platformer controller (jump height and time to apex,
    coyote time, jump buffer, apex hang, jump cut, air and wall jumps,
    one-way platforms with drop-through, corner correction) with
    `og.CharacterEvent`, and vehicles (rapier's raycast car in 3D, sprung
    motorised wheel joints in 2D).
  - Also new: Verlet ropes and cloth, a 2D particle fluid (double density
    relaxation), height-field and 2D spring-line water with splashes and
    buoyancy, and ragdolls built from an og-anim skeleton and blended from
    the animation.
  - All of it is inside the og.physics state block. A 3D and a 2D world
    with every feature hash the same twice and after a restore mid-run, and
    the hashes are bit-identical on aarch64 macOS and x86_64 Linux.
  - **og-render** gains fog (distance and height, sun scattering),
    procedural sky looks drawn per pixel, `day_night` and weather presets
    (rain, snow, dust, mist).
  - It gains planar water (depth colour, foam, refraction, screen-space
    reflections on the full tier and the sky cubemap on WebGL2), 2D water
    strips and metaball fluids in the sprite order.
  - It gains material shader hooks (vertex offset, surface colour, normal,
    emissive), which og-cook compiles with naga for every back end and
    refuses with naga's message, and mesh replacement.
  - All of it is off by default, and every earlier reference frame is
    unchanged.
  - **TypeScript** reaches it through the IDL's physics module, the new
    atmosphere module (`og.Fog`, `og.Sky`, `og.DayNight`, `og.Weather`,
    `og.Water`, `og.Water2D`, `timeOfDay`), `og.render.shaderMaterial` and
    `setShaderParam`, and `og.load.shader`.
  - **Tests and references**: `templates/fx-showcase` has eight new
    stations, each with its playtest and references on `metal`,
    `vulkan-cpu`, `webgpu` and `webgl2`. og-tests has nine new scenes with
    `metal` and `vulkan-cpu` references and behaviour tests.
  - **Knowledge**: og-game (atmosphere and water, material shaders, physics
    beyond rigid bodies), og-feel §12 (controller tunables by genre) and
    og-taste §8 (atmosphere by intent).
  - **Cost** on the M4 Pro: water 0.66 ms per frame. The 2D fluid costs
    1.15 ms per tick at 1,000 particles. Every other feature costs under
    0.12 ms.
  - **Wasm**: 2,664,999 bytes brotli, 63 % of the budget (60 % before),
    with no feature behind a cargo flag.
  - Evidence: `tests/evidence/E5/`.
- Owed: §Owed rows 7–8 and 23–29.

### Status 2026-10-18, E3 round

- **UI style** (d612bc3..24d3663): og-ui's skin layer: a `Skin` the theme
  carries (nine-slice frames per element class from one atlas, corner,
  divider and heading ornaments, panel tiles, shadows and glows per class,
  text shadows and glows per font role, press squash and hover lift, fonts per
  role with a CJK fallback, an icon set, motion and sounds), cooked from
  `.ogt` files of `type skin` (`og.UiSkin` blobs) and loaded with
  `og.load.skin`; `UiDesc.skin` and `sounds`, `og.ui.counter`,
  `og.ui.burst`, `og.ui.screenTransition` (fade, wipe, iris, dissolve,
  slide; colour or texture, over every layer through the overlay pass),
  `TransitionKind.pop`, `ElementDesc.frame`, `LabelDesc.heading`; UI events
  sound through og-audio on the ui bus (`og.ui.sound`). The default skin
  dresses nothing: every token reference frame is unchanged and the
  captures were pixel-identical before and after. Six style packs (pixel,
  cartoon, fantasy, scifi, paper, neon) in `templates/ui-kit/assets/skins/`
  with generated atlases and icons and OFL fonts, held to 4.5:1 by
  `og-ui/tests/packs.rs`; the kit's 20 screens in every pack against one
  metal sheet per pack; the og-taste skill. Full gate at 24d3663 in 4 min 2 s
  (wasm 2,428,536 bytes brotli, 57 % of the budget; +59,841 over E1).
  Evidence `tests/evidence/E3/`. Owed: §Owed rows 1–3 and 15–20.

### Toolchain 2026-10

- **2026-10-19** (c1290aa..HEAD): Rust 1.97.0 to 1.99.1 with every crate
  on edition 2024 (clippy's new as_chunks and let-chain lints applied),
  binaryen 123 to 133 plus `--low-memory-unused`, wasm-bindgen 0.2.127 to
  0.2.129, TypeScript 5.9.3 to 7.0.2 (0.26 s instead of 1.2 s for the
  engine typecheck), cargo-nextest, cargo-zigbuild and zig newer locally.
  The shipping wasm is 1.0 % smaller (2,518,068 bytes brotli, 60 % of the
  budget); no reference frame or determinism number moved. The template's
  LFS attributes are stored as `gitattributes` and renamed by the
  scaffolder, so this repository's checkouts are clean again; fx-showcase's
  references on all four tiers carry the one-line badge of 00653bf. wgpu
  31 is not published yet (§Owed row 35). Versions: `docs/ci.md` §Toolchain; evidence
  `tests/evidence/toolchain-2026-10.md`.

### Build speed 2026-10

- **2026-10-19** (edd9736..HEAD): the inner loop, measured with another
  agent on the machine. The full lane went from 165–320 s to 103 s with
  `--all` and 41 s after a template-only change, the quick lane from
  83–154 s to 19–26 s, `cargo nextest run --workspace` from 112 s to
  20–29 s. The causes were not the compiler: 252,881 stale object files
  in `target/debug/deps` that every Metal device open read (1.4 s per GPU
  test; the dev profile now has no debuginfo), og-cook's tests cooking the
  template once per process and one at a time (now once per test build,
  under a file lock, in parallel), an O(n²) vertex search, and nextest
  starting the longest work last. og-api is three crates (og-idl, og-api,
  og-runtime), so a doc-only IDL edit rebuilds nothing; the gate is
  path-aware (GPU tests, wasm steps, drift, web-reference skipped when
  their inputs did not change; `--all` and CI run everything), wasm-opt
  and brotli reuse results by content, sccache is the wrapper in
  `.cargo/config.toml` (a fresh worktree's test build about 70 s) and
  parallel rounds reuse `scripts/worktree.sh` worktrees. No reference frame
  was re-recorded. Rules: `docs/ci.md` §Lanes, AGENTS.md §Build and gate
  discipline; evidence `tests/evidence/build-speed-2026-10.md`.

### Status 2026-10-18, E4 round

- **2D and game feel** (b6da6a1..4e63a59, branch `e4-2d-feel`): `og.TimeScale`
  in every world (`og.time.setScale`, `og.time.hitStop`; physics, animation,
  flipbooks and scaled tweens step by `scaledDt`, systems, the UI and the
  probe keep ticking); og-render's 2D batch sorted by layer then y, with
  nearest textures, nine-slice, chunked tile layers, text outlines and
  layered trails; `og.Sprite` (y sorting, flips, flash, blob shadow,
  nine-slice) and `og.Afterimage`; og-anim flipbooks (grid, named or
  Aseprite / TexturePacker atlas sheets, clips with fps, loop / pingPong /
  once, speed, `og.FlipbookEvent`) and `texture { kind pixel }`; the new
  og-tween (every numeric field, 22 easings, delay, loops, yoyo, sequence
  and parallel, `og.TweenCompleted`, squash and flash presets) and
  og-tilemap (tilesets with collision and animated tiles, 47-blob and
  16-bitmask auto-tiling, chunked layers with parallax and repeat, merged
  static colliders, deterministic `setTile` edits that re-tile their
  neighbours) with Tiled `.tmj` and LDtk `.ldtk` importers in og-cook;
  camera feel (trauma shake, punch, kick, Follow2D dead zone, look-ahead and
  per-axis damping, `og.CameraZone`, `og.CameraRail`; frame-only) and
  `og.render.floatingText`. Every piece has a state block, so snapshots
  replay. The archetypes `runner` and `match3` are rebuilt on it and
  `topdown` (Tomb Room) is new; each has frames and replay playtests with
  `metal` and `vulkan-cpu` references (topdown also `webgpu` and `webgl2`
  through the generalised web reference runner). A 256×256 four-layer map
  draws in 0.22 ms at 1280×720 on the M4 Pro (budget 1 ms). Fixed on the
  way: a scene component's omitted fields cooked as zero instead of the IDL
  default, presets started with a hit stop showing nothing until it ended,
  and a sheet's `filter linear` ignored on a pixel texture. Knowledge: the
  og-feel skill and og-game's 2D and feel sections. Evidence
  `tests/evidence/E4/`. Owed: §Owed rows 4–6 and 39.

### Status 2026-10-18, E2 round

- **Render and Runtime** (96adafe..393c2d6): every E1 effect is declared
  from TypeScript on both hosts (`og.PostEffects`, `og.Lighting2D`,
  `og.Light2D`, `og.Occluder2D`, `og.Decal`, `og.ParticleSystem`,
  `og.Trail`, `og.Beam`, `og.render.ribbon`, `og.load.lut`,
  `og.load.emitter`, sprite and material effect fields; off by default);
  og-render's screen-space overlay pass with per-pixel clips, which og-ui
  now draws through (no camera plane, no CPU clipping, sharp under TAA);
  `og.Pointers` (five pointers) and `og.Gamepad` (four pads) in og-input
  through the input ring, read by og-ui's touch controls and focus
  navigation; `templates/fx-showcase` with five stations and references on
  `metal`, `vulkan-cpu`, `webgpu` and `webgl2` (the browser tiers from
  `make web-reference`, each within og-tests' tolerance of `metal`,
  docs/ci.md §Browser reference tiers). Fixed on the way: immediate-mode
  sprites and text outliving their tick, effect passes that aliased the
  prepass textures they bind, dissolved pixels drawn black on Metal, the
  overlay blending encoded sRGB on the WebGPU canvas, three WebGL2 defects
  (TAA's depth read, 18 of 16 GL texture slots in the lit skinned
  pipeline, the local shadow map created as a cube-map array) and the
  Worker's probe call trapping the wasm on a non-string argument.
  Evidence `tests/evidence/E2/`. Owed: §Owed rows 20–22.

### Status 2026-10-18, E1 rounds

After M7 (the platform cutover, `docs/m7-platform-notes.md`) the E1 rounds
ran as four parallel streams on one tree, gated by `scripts/check.sh` end
to end (wasm 2,368,695 bytes brotli, 56% of the 4 MB budget; 1,099,937 at
M6: og-anim, og-ui, og-physics and the effects are the growth).

- **Animation** (0790651): og-anim and the cook's skin/clip/morph import;
  the `anim` IDL module (`og.load.animation`, `og.anim`) on both hosts
  with the walker posed identically on the native addon and in wasm
  (`packages/engine/tests/anim.test.ts`). Evidence `tests/evidence/E1/anim-*`.
- **UI kit**: about thirty widgets, focus navigation, motion, breakpoints,
  theme variants and UI state format 2 in og-ui; nineteen screen templates
  and the toast service in `og.ui` (103 functions); the `templates/ui-kit`
  gallery with 42 metal references at desktop and phone sizes and a
  two-host hash match in Chrome (WebGPU); the `og-ui` skill. Evidence
  `tests/evidence/E1/ui-*`. Owed then: the og-render overlay pass with
  per-instance clips, a gamepad resource and multi-touch in og-input (all
  delivered in E2, above).
- **Render effects**: AO, outlines, depth of field, motion blur, god rays,
  LUT grading, vignette, aberration, grain, flash, camera shake, decals,
  deterministic GPU/CPU particles, trails, ribbons and beams, 2D lights
  with soft shadows and glow, per-sprite effects, GPU skinning and
  blendshapes, toon/rim/dissolve (`crates/og-render/README.md` §E1
  effects). Fourteen reference scenes on `metal` and `vulkan-cpu`, 25
  effects tests, features-off references byte-identical. Costs
  `tests/evidence/E1/vfx-feature-costs.md`. Owed: the overlay pass (delivered in E2) and
  browser runs of the effect scenes (§Owed row 22).
- **Web export** (`docs/web-export.md`): project.ogt's `publish` block,
  `og-build.json` with every file's bytes and sha256, `_headers`,
  `origingame.runtime.json`, the PWA manifest and service worker, byte-
  deterministic output; the shell's splash, gate, error card and install
  hint; the og-probe wire for Stage and Player parents (request ring when
  isolated, postMessage otherwise) and the Player protocol 2; gamepad
  input; `og run` reports a dying game process at once; `og doctor`
  outside a project. Browser evidence `tests/evidence/E1/web-*`.
- **Still pending**: §Owed rows 38 and 40.

### Status 2026-10-18, M6

The M6 crates are committed (8d160fe render, 4fb20c7 ui, 55ac8a2 audio,
5c79c75 tools, d214078 ECS state blocks, 49ed935 physics) and
`scripts/check.sh` passes end to end at 49ed935 (all nine steps; wasm
1,099,937 bytes brotli of the 4 MB budget). What each crate's README owed
then has since landed or is in §Owed: the overlay pass and the `ui` module
in the IDL (E1, E2), og-audio's host wiring, effects and `listen` (E6),
og-physics in the IDL with its controllers (E5), the depth prepass (E1's
effects), the archetypes (E4) and the template's playtest starting on its
Start button (E6); still open are §Owed rows 30–34 and the mobile
measurements, row 40 (§5: scheduled for week 9, they need devices).

## 3. The first two weeks, concretely

Week 1 (M0, integrator + all four):

1. Workspace: create the crates with `lib.rs` stubs and the dependency graph
   of design §Repository layout; `Cargo.toml` pins (wgpu 30, glam 0.34,
   rapier 0.2x, gltf 1.4, meshopt, basis-universal, naga); `rust-toolchain.toml`.
2. CI (GitHub Actions): `cargo fmt --check`, `clippy -D warnings`, `cargo
   test`, wasm lib build + `wasm-opt` + brotli size gate (fail above 4 MB),
   generated-file drift check, `bun test` for the TypeScript packages.
3. `tests/`: the gfx harness (render a scene headless, compare to a committed
   PNG with a per-pixel tolerance and an RMSE threshold, write the diff on
   failure), the perf harness (frame-time baseline files with tolerance).
4. Move the spike renderer into `og-render` behind `Renderer::new(&RenderDesc)`
   and `render(&World)`; keep the spike's reference frame as the first gfx test.
5. Write `api.ogt` v1 as text (Core) while Render and Assets start their
   crates; the integrator reviews it against the design's §3.2 example.

Week 2 (M1 begins, M2 generators begin):

1. Core: the `Reflect` derive and schema JSON; the `.ogt` grammar and parser
   with round-trip tests on the design's examples.
2. Core: archetype storage and queries with the 10k-churn test.
3. Tools: the IDL parser and the first generator (`.d.ts`), driven by
   `api.ogt`; the `og` CLI skeleton (`og --version`, `og doctor`).
4. Render: material assets from reflected structs; automatic instancing keyed
   by (mesh, material); the sky pass and mip generation.
5. Assets: glTF → meshopt-encoded vertex streams as the first `.ogb` payload.

## 4. How the work is run

- Each workstream is one long-lived agent thread on its own crates with the
  IDL and this document as its brief; cross-crate changes go through the
  integrator's thread. The platform's own thread family is the natural tool
  for this once the repository is registered with Agent Service; until then
  the threads run from the developer's machine.
- Weekly: integrator merges, runs the full gfx and perf suites, updates this
  document's milestone table with evidence links, and publishes a
  `0.0.x` of `@origingame/engine` to the platform's internal registry so the
  M7 integration can start against real packages early.
- Every milestone's evidence is a file in the repository (`tests/evidence/
  M<n>/`), never a chat message.
- The platform repository changes nothing until M7, except
  `goals/origin-engine/plan.md` recording progress. The Godot distribution
  and the Three.js substrate keep shipping meanwhile.

## 5. Risks specific to phase 1

| Risk | Mitigation |
|---|---|
| The IDL generators become the bottleneck for every workstream | the `.d.ts` generator ships in week 2 and the two host generators by week 4; until then hosts are hand-written against the IDL and replaced, not extended |
| Renderer scope creep (every feature in §2.6 at once) | M3's 12 reference scenes are the list; anything not in a scene waits for phase 2 |
| rapier's wasm size pushes past the budget | measured in M6 with the size gate; the fallback is a feature-gated physics crate loaded only by projects that declare physics in `project.ogt` |
| Basis encoder build complexity in the cook | `basis-universal` crate first; if its build is fragile on CI, `toktx` as an external tool found by `og doctor` |
| Mobile numbers slip again for lack of devices | M6 names them as evidence; the integrator schedules the device run in week 9, not week 13 |
