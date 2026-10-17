# Origin Engine

The game engine of the OriginGame platform: a Rust + wgpu core that runs as
WebAssembly in the player's browser and as a native binary in headless
executors, driven by game code written in TypeScript. It is built for one
author above all others: an AI agent that has to write, run, observe and
correct a game in minutes without a human at the editor.

Status: **spike done, phase 1 starting** (`docs/design.md`, results in `crates/og-spike/RESULTS.md`, execution in `docs/phase-1.md`). The
platform-side execution package is `origingame/goals/origin-engine/plan.md`.

## What it is, in one paragraph

A game is a TypeScript package against `@origingame/engine`. The engine owns
the world (an ECS with reflected components), a fixed deterministic tick, the
renderer (WebGPU, WebGL2 fallback), animation, physics, navigation, audio,
UI, the asset cook and the probe. Game code declares components and systems
and reads the world through typed views over engine memory, so per-frame work
stays inside the engine. The same engine builds as `origin-engine.wasm` for
browsers and as a native Node addon for the sandbox, where `og` runs the same
TypeScript game headless and captures frames through Mesa or a GPU. Every
game has a command channel and a probe from its first frame: scripted input,
screenshots, world dumps, parameter sweeps and deterministic replay are
engine features, not tooling bolted on afterwards.

## Repository layout (planned)

```
crates/           Rust workspace: the engine core and its backends
  og-api          the IDL (`api.ogt`) and the generators: Rust traits, .d.ts, wiki, JSON schema
  og-core         allocators, containers, math (glam), ids, reflection, text formats
  og-ecs          world, archetypes, queries, schedule, snapshot/restore
  og-render       wgpu renderer: forward+, PBR, shadows, IBL, post, text, particles
  og-anim         skeletal animation, blend trees, IK
  og-physics      rapier3d integration, character controller
  og-nav          navmesh build and query
  og-audio        mixer, buses, spatialisation (AudioWorklet on web)
  og-ui           immediate-mode UI on the OriginGame design tokens
  og-cook         asset cook: glTF, textures (KTX2/Basis), audio, fonts (MSDF), scenes
  og-probe        command channel, probe RPC, dumps, capture, sweep
  og-host-web     wasm-bindgen bindings and the Worker/OffscreenCanvas host
  og-host-node    napi bindings for the headless native player
  og-cli          the `og` tool: build, run, cook, probe, doctor, test
packages/
  engine          npm `@origingame/engine`: wasm, native addon, .d.ts, the TS API
  create          `npm create @origingame/game`: templates and archetypes
  knowledge       skills, wiki generation, scaffold (consumed by the platform's game-knowledge)
shaders/          WGSL sources, cooked per backend at build time
templates/        the project template and the Discover archetypes
tests/            logic, multicore, gfx (reference frames), perf (baselines)
docs/             design.md (normative), wiki/ (generated from API docs), decisions
```

## Commands (planned)

```
og build            build the game (bun bundle + cook) for the host platform
og run              build and run; `--commands "t:action:args;..."` scripts a run
og cook             cook assets into cooked/
og probe <method>   call the running game's probe
og test             run the project's tests; `--capture` records gfx references
og doctor           engine, toolchain, project and knowledge checks, with fixes
```

Read `AGENTS.md` before changing anything.
