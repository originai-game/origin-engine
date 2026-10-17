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
| **Core** | `og-core`, `og-ecs`, `og-api` | ids, handles, reflection macro and schema, `.ogt`/`.ogb`, world, archetypes, queries, schedule, snapshot/restore, deterministic rng and math, the IDL and its generators |
| **Render** | `og-render`, `shaders/` | render graph, materials, instancing, lights, CSM, IBL, sky, mips, MSAA/TAA, bloom/tonemap, 2D sprite and text batches, capture readback |
| **Assets** | `og-cook`, `templates/` | glTF import (meshopt, LODs, tangents), KTX2/Basis, audio (QOA), MSDF fonts, scene compile, cook stamps, the project template and archetypes |
| **Tools** | `og-probe`, `og-cli`, `og-host-web`, `og-host-node`, `packages/engine`, `packages/create`, `packages/knowledge` | command channel, probe, dumps, `og` (build/run/cook/probe/test/doctor), the two hosts, the npm packages, wiki generation |
| **Integrator** | workspace, CI, `tests/`, `docs/` | IDL stewardship, weekly merges, gfx/perf suites, templates in sync, the milestone evidence |

## 2. Milestones

| # | Weeks | Deliverable | Evidence |
|---|---|---|---|
| M0 Scaffold | 1 | workspace with every crate stubbed, CI (fmt, clippy, test, wasm build, size gate, generated-file drift), `og` skeleton, `tests/` harness with gfx reference compare, the spike renderer moved into `og-render` behind the new crate boundary | CI green on an empty game; the spike frame reproduced from `og-render` byte-identical to `.spike-out/native.png` |
| M1 World | 1–3 | `og-core`: `#[derive(Reflect)]` → schema JSON, `.ogt` reader/writer with round-trip, handles, rng, math; `og-ecs`: archetype storage, queries, system schedule, resources, events, snapshot/restore | logic tests: 10k-entity structural churn, snapshot→restore→identical world hash, `.ogt` round-trip on every template file; a Rust-only "bouncing boxes" world advanced 3600 ticks twice gives the same hash |
| M2 API | 2–4 | `og-api`: `api.ogt` v1 (world, components, systems, resources, transform, camera, input, time, rng, load, probe), generators for Rust traits, wasm-bindgen glue, napi glue, `.d.ts`, JSON schema, wiki pages; typed views over columns with tick-stamped invalidation; `og-host-web` Worker + OffscreenCanvas; `og-host-node` | a TypeScript hello-world (spawn 10k entities, one `Move` system) runs identically on both hosts: same world hash after 600 ticks; the spike's view benchmark re-run through the real API within 2× of the raw loop |
| M3 Renderer | 3–6 | render graph; material system from reflected material assets; automatic instancing by (mesh, material); directional + point/spot lights; 3-cascade CSM; IBL + sky pass; mip generation; MSAA (GL) / TAA (WebGPU); bloom, tonemap, exposure; 2D sprite batch and MSDF text; capture readback on both hosts | gfx reference frames for 12 scenes (committed PNGs, per backend tier); the spike's glTF scene within ±10 % of the spike's frame time; 20k moving sprites ≤ 2 ms on the M4 Pro |
| M4 Cook | 5–8 | `og cook`: glTF → `.ogb` (meshopt vertex/index codec, cache ordering, LOD chain, tangents), textures → KTX2 Basis (UASTC for normals, ETC1S for colour), audio → QOA, fonts → MSDF atlas, scenes `.ogt` → `.ogb`; incremental by hash; cook stamps checked at load; runtime loaders; hot reload by snapshot + bundle swap | cook of the template and archetypes is deterministic (same bytes twice); a glTF edit → `og cook` → reload shows the change in ≤ 300 ms with world state kept; cooked size of the template ≤ 60 % of its sources |
| M5 Probe | 6–9 | `og-probe`: command channel (core actions of design §3.3), module-registered commands, probe protocol 5 members + `capture`, `sweep`, `reload`, `snapshot`, `restore`, `dump`, `commands`; native loopback WebSocket; in-page request ring; `publish` trim | the platform's probe conformance suite (`origin/packages/contracts`) green on both hosts; `og run --commands "0.5:touch_move_to:Start;0.6:touch_start;0.7:touch_end;2.0:screenshot;2.2:dump_world;2.5:quit"` on the template produces the frame and the dump; a `sweep` over a tunable yields 8 frames in ≤ 1 s native |
| M6 Games | 8–11 | input (pointer, keys, gamepad, touch), camera controllers, audio (AudioWorklet ring, cpal), 2D physics (rapier2d) and 3D rigid bodies (rapier3d, static + dynamic, no character controller yet), minimal UI (panels, buttons, text, stable ids), project template, `npm create @origingame/game`, two archetypes rebuilt (`runner` 2D, `match3`), wiki generated, skills and scaffold in `packages/knowledge` | both archetypes playable in a browser and through `og run --commands`; their playtest scripts pass headless; the knowledge package validates against the platform's `packages/game-knowledge` schema; first-frame on an iPhone and the Android view benchmark recorded |
| M7 Platform | 11–13 | `@origingame/engine` published (wasm + native addon per platform via optionalDependencies, `.d.ts`); E2B `engine` class image (bun + engine + lavapipe); runner `og.rs` for `stage_capture`/`stage_record`/`level_report`; mode table without substrate; one Discover prototype produced end to end by a thread on the new engine | the thread's session transcript with frames; `engine` image ≤ 600 MB; `bun deploy/ovh/state.ts` shows the new class live; the platform goal's phase-1 row filled in |

Dependencies: M1 → M2 → everything TypeScript-facing; M0 → M3 (renderer
work starts from the spike immediately); M4 needs M1's `.ogb`; M5 needs M2's
hosts; M6 needs M3, M4, M5; M7 needs M6.

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
