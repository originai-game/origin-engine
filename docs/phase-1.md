# Phase 1 — execution plan

Scope: `docs/design.md` §11 phase 1 ("Core"), 13 weeks from 2026-10-20.
Output: a game author can `npm create @origingame/game`, write TypeScript
against `@origingame/engine`, run it in a browser and headless with `og`, and
the platform can run it in an `engine`-class sandbox. Spike findings
(`docs/spike-results.md`) are the baseline; the spike crate is the seed
of `og-render` and is deleted once its code has moved.

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
| M3 Renderer | 3–6 | render graph; material system from reflected material assets; automatic instancing by (mesh, material); directional + point/spot lights; 3-cascade CSM; IBL + sky pass; mip generation; MSAA (GL) / TAA (WebGPU); bloom, tonemap, exposure; 2D sprite batch and MSDF text; capture readback on both hosts | gfx reference frames for 12 scenes (committed PNGs, per backend tier); the spike's glTF scene within ±10 % of the spike's frame time; 20k moving sprites ≤ 2 ms on the M4 Pro. **Filed** (`tests/evidence/M3/`, 2026-10-17): `showcase-metal.png`, `showcase-vulkan-cpu.png`, `perf-baselines.json`, `feature-costs.md`. 11 Metal and 11 vulkan-cpu references are committed (`crates/og-tests/references/`); the `msaa` scene and the GL/WebGL2 tier have none. |
| M4 Cook | 5–8 | `og cook`: glTF → `.ogb` (meshopt vertex/index codec, cache ordering, LOD chain, tangents), textures → KTX2 Basis (UASTC for normals, ETC1S for colour), audio → QOA, fonts → MSDF atlas, scenes `.ogt` → `.ogb`; incremental by hash; cook stamps checked at load; runtime loaders; hot reload by snapshot + bundle swap | cook of the template and archetypes is deterministic (same bytes twice); a glTF edit → `og cook` → reload shows the change in ≤ 300 ms with world state kept; cooked size of the template ≤ 60 % of its sources. **Filed** (`tests/evidence/M4/`, 2026-10-17): `cook-determinism.json`, `cook-report-template.md` (cooked 35.7 % of sources), `cooked-gltf-reference.png`, `cooked-gltf-reference-compare.json`, `texture-psnr.md`. Not filed: the glTF edit → reload ≤ 300 ms measurement (`og dev` prints it since 5c79c75). |
| M5 Probe | 6–9 | `og-probe`: command channel (core actions of design §3.3), module-registered commands, probe protocol 5 members + `capture`, `sweep`, `reload`, `snapshot`, `restore`, `dump`, `commands`; native loopback WebSocket; in-page request ring; `publish` trim | the platform's probe conformance suite (`origin/packages/contracts`) green on both hosts; `og run --commands "0.5:touch_move_to:Start;0.6:touch_start;0.7:touch_end;2.0:screenshot;2.2:dump_world;2.5:quit"` on the template produces the frame and the dump; a `sweep` over a tunable yields 8 frames in ≤ 1 s native. **Filed** (`tests/evidence/M5/`, 2026-10-17): `probe-conformance-test-output.md`, `probe-conformance-transcript.json`, `scripted-run-dump-world.json`. Not filed: the `og run --commands` run on the template (see M6 `tools-template-*`) and the timed 8-frame `sweep`. |
| M6 Games | 8–11 | input (pointer, keys, gamepad, touch), camera controllers, audio (AudioWorklet ring, cpal), 2D physics (rapier2d) and 3D rigid bodies (rapier3d, static + dynamic, no character controller yet), minimal UI (panels, buttons, text, stable ids), project template, `npm create @origingame/game`, two archetypes rebuilt (`runner` 2D, `match3`), wiki generated, skills and scaffold in `packages/knowledge` | both archetypes playable in a browser and through `og run --commands`; their playtest scripts pass headless; the knowledge package validates against the platform's `packages/game-knowledge` schema; first-frame on an iPhone and the Android view benchmark recorded. **Filed so far** (`tests/evidence/M6/`, 2026-10-17 UTC; partial, M6 is not done): Core `core-cross-target-determinism.md`, `core-input-edges-transcript.json`, `core-break-and-restore.md`; Render `render-showcase-metal.png`, `render-showcase-vulkan-cpu.png`, `render-shadow-map-256.png`, `render-shadow-map-2048.png`, `render-texture-formats.md`, `render-test-run.md`, `render-feature-costs.md`, `render-perf.md`; UI `ui-sample-screen-metal.png`, `ui-dump-ui-sample.json`, `ui-test-run.md`; Audio `audio-determinism.json`, `audio-mix-timing-release.json`, `audio-mix-timing-debug.json`, `audio-scene-waveform.png`, `audio-test-run.md`; ECS `ecs-state-blocks.md`; Physics `physics-pile-determinism.json`, `physics-pile-determinism-debug.json`, `physics-wasm-size.json`, `physics-test-run.md`; Tools `tools-template-run-frame.png`, `tools-template-dump-world.json`, `tools-template-dump-entity.json`, `tools-template-run.md`; gate `check-summary.md`, `size-gate.json`. Not yet: the archetypes, their playtests, the knowledge package against the live platform schema, the mobile numbers. |
| M7 Platform | 11–13 | `@origingame/engine` published (wasm + native addon per platform via optionalDependencies, `.d.ts`); E2B `engine` class image (bun + engine + lavapipe); runner `og.rs` for `stage_capture`/`stage_record`/`level_report`; mode table without substrate; one Discover prototype produced end to end by a thread on the new engine | the thread's session transcript with frames; `engine` image ≤ 600 MB; `bun deploy/ovh/state.ts` shows the new class live; the platform goal's phase-1 row filled in |

Dependencies: M1 → M2 → everything TypeScript-facing; M0 → M3 (renderer
work starts from the spike immediately); M4 needs M1's `.ogb`; M5 needs M2's
hosts; M6 needs M3, M4, M5; M7 needs M6.

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
  Evidence `tests/evidence/E3/`. Owed: phone and `vulkan-cpu`/browser
  references for the packs (metal desktop only); per-glyph tracking from a
  skin's ramp and a real text glow (an MSDF outline in og-render); tiled
  (not stretched) nine-slice edges; springs for hover and press; the UI
  sounds start one tick after the input (`og.ui.sound` runs after
  `og.audio`); skins drawn by hand for a game still need an atlas tool
  beyond `make-skins.py`.

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
  31 is not published yet. Versions: `docs/ci.md` §Toolchain; evidence
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
  `tests/evidence/E4/`. Owed: no third native tier exists (only `metal`
  and `vulkan-cpu`; the browser tiers stand in for topdown); game
  commands cannot read or set resources; camera zones clamp the frame
  without scrolling between rooms; `getTile` on a terrain layer answers
  the terrain bit, not the resolved tile.

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
  Evidence `tests/evidence/E2/`. Owed: browser runs of og-tests' own gfx
  scenes (they need a wasm harness), `vulkan-cpu` references for og-ui
  and the UI kit, pads beyond the first in og-ui.

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
  `tests/evidence/E1/vfx-feature-costs.md`. Owed: the browser-tier
  references for the effect scenes, and an overlay pass (above).
- **Web export** (`docs/web-export.md`): project.ogt's `publish` block,
  `og-build.json` with every file's bytes and sha256, `_headers`,
  `origingame.runtime.json`, the PWA manifest and service worker, byte-
  deterministic output; the shell's splash, gate, error card and install
  hint; the og-probe wire for Stage and Player parents (request ring when
  isolated, postMessage otherwise) and the Player protocol 2; gamepad
  input; `og run` reports a dying game process at once; `og doctor`
  outside a project. Browser evidence `tests/evidence/E1/web-*`.
- **Still pending**: the mobile measurements below (devices), and the
  platform side of the cutover (`origin-engine-cutover` branch in the
  platform repository: merge, then pin this revision).

### Status 2026-10-18, M6

The M6 crates are committed (8d160fe render, 4fb20c7 ui, 55ac8a2 audio,
5c79c75 tools, d214078 ECS state blocks, 49ed935 physics) and
`scripts/check.sh` passes end to end at 49ed935 (all nine steps; wasm
1,099,937 bytes brotli of the 4 MB budget). What each still owes, by its
README:

- **og-render** (§What phase 2 owes): "a depth prepass; Forward+ clustered
  lights"; "GL-tier references … a committed browser runner for `webgpu`
  and `webgl2` is still owed, and with it a run of `Renderer::capture` in a
  browser" (both since delivered for fx-showcase: `make web-reference`, see
  the E2 status above); "a normal matrix for non-uniform scale; vertex tangents;
  clearcoat"; "screen-space UI layering (sprites are world-space today)";
  the runtime loaders do not upload the tier's compressed format yet. No
  gfx reference scene shows the M6 material features.
- **og-ui** (§What phase 2 owes): "The og-render overlay and per-instance
  clip above; then drop CPU clipping"; the `ui` module is not in
  `api.ogt` and no host links og-ui, so games cannot call it; rich text,
  animations, accessibility, glass and layout extensions are phase 2.
- **og-audio** (§Status): "the wiring listed under §Host contract is not
  done: the `og-api` runtime does not `install` og-audio or publish the
  link, `og run` and the web shell/Worker open no sink, and `api.ogt` has
  `load.sound`/`Sound` but no `AudioSource`/`AudioListener`/`Audio`
  records"; effects, occlusion, music transitions and probe audio capture
  are phase 2.
- **og-physics** (§Status): not in `api.ogt` and not linked by either host;
  no character controller (this table's M6 row defers it). Its wasm cost is
  measured: 694 KB brotli for 3D, 541 KB for 2D, 1.05 MB for both
  (`tests/evidence/M6/physics-wasm-size.json`), inside the 4 MB budget
  without the §5 feature-gating fallback.
- **og-cook** (§What phase 2 owes): "the runtime loaders transcode to the
  format `RenderInfo::texture_formats` names and upload it"; "Archetype
  templates (`runner`, `match3`) and their assets"; the M4 hot-reload
  measurement is not filed.
- **og-ecs**: state blocks are done; its README lists nothing owed.
- **Tools**: the template run passes headless (`tools-template-run.md`),
  but its frame shows no UI (the playtest starts through the `s` key) and
  the world-space label is partly hidden behind the dropship.
- **Pending mobile measurements** (this row's evidence and
  `docs/spike-results.md`): the iPhone first frame and frame times (Safari,
  WebGPU) and the mid-range Android run of the view benchmark
  (`bench/views.ts`, ≤ 1.0 ms per tick for 10,000 entities); neither is
  recorded, both need devices (§5: scheduled for week 9).

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
