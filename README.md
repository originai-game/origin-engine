# Origin Engine

The game engine of the OriginGame platform: a Rust + wgpu core that runs as
WebAssembly in the player's browser and as a native binary in headless
executors, driven by game code written in TypeScript. It is built for one
author above all others: an AI agent that has to write, run, observe and
correct a game in minutes without a human at the editor.

- Platform: [origingame.ai](https://origingame.ai)
- Engine: [engine.origingame.ai](https://engine.origingame.ai)

## What it is, in one paragraph

A game is a TypeScript package against `@origingame/engine`. The engine owns
the world (an ECS with reflected components), a fixed deterministic tick, the
renderer (WebGPU), animation, physics, navigation, audio,
UI, the asset cook and the probe. Game code declares components and systems
and reads the world through typed views over engine memory, so per-frame work
stays inside the engine. The same engine builds as `origin-engine.wasm` for
browsers and as a native Node addon for the sandbox, where `og` runs the same
TypeScript game headless and captures frames on a GPU. Every
game has a command channel and a probe from its first frame: scripted input,
screenshots, world dumps, parameter sweeps and deterministic replay are
engine features, not tooling bolted on afterwards.

## Design pillars

Normative detail is in `docs/design.md` (and `docs/ui.md` for the UI).

- **One API, two hosts.** The TypeScript API is generated from one IDL
  (`crates/og-api/api/<module>.ogt`) for the wasm host (WebGPU, in a Worker on
  an OffscreenCanvas) and the native napi host (headless, offscreen). Backend
  limits are capability queries, never missing members.
- **Deterministic by construction.** Fixed tick, engine-owned seeded
  randomness, no wall-clock reads in the tick, snapshot and restore of the
  whole world, a portable world hash, record and replay (`og replay`).
  Rendering never feeds back into simulation.
- **Observable from the first frame.** The command channel, the probe
  (RuntimeProbe protocol 5), `dump_world`/`dump_entity`/`dump_ui` and frame
  capture exist in every build except `publish`, which keeps only the
  player-facing probe members.
- **Reflection is the single schema.** Components are declared once; the
  inspector, Tune, the probe, scene text and the wiki read that schema.
- **Text in, blobs out.** Scenes and assets are line-based `.ogt` text, cooked
  into `.ogb` blobs with a type hash, a schema version and a cook stamp.
- **Built for an agent author.** Desc structs whose zero value is a sensible
  default, errors as values, three.js vocabulary, a generated wiki and skills
  shipped in the package, and `og` commands that check what a reviewer checks
  by eye (`og polish`, `og ui check`, `og style check`).

## Status (2026-11-03)

Features first, compatibility last. Released: 0.7.7 on npm (2026-10-31).
The work on main follows `docs/roadmap.md`: the UI wave's controls,
containers, feedback/HUD, patterns and screen documents (U6) and waves G1 to
G3 have landed; the realism wave is mostly on main (R4 to R7 and R9 to R12
done). In progress: R1 (meshlets, culling and the visibility buffer are on
main; the stream is open), R2 with R8 (TAAU is on main; more cascades,
shadowed local lights, contact shadows, lens flare, SMAA, HDR output and the
environment-map gaps are not), R3 (screen-space GI is on main; probes
updated at run time are not), U8 (the kit's genre showcases, every seed style
on them) and the og-ui gaps stream. Metal on macrunner is the only pixel
reference for now; DX12, the browsers and phones are a compatibility phase
after the features. Deferred: accessibility, the genre archetypes, and
performance and size, which are measured at each wave's end.

## Features

What is on main, grouped by area, with the API module (`og.<module>`, from
`crates/og-api/api/`) and the crate that implements it.

**Rendering** (`og.render`, `og.atmosphere`, `og.terrain`, `og.debug`; og-render)
- Forward PBR with three.js material parameters, shadows, IBL and environment
  maps, reflection probes, SSR, area (rect) lights, volumetric fog, bloom and
  post effects; TAA, MSAA and FXAA; temporal upscaling (TAAU) with render
  scale and dynamic resolution; frame cap and automatic quality.
- Meshlets: clusters cut by the cook, GPU frustum, normal-cone and two-phase
  occlusion culling, indirect draws, shadows through the same path, a
  visibility buffer with compute rasterisation of small clusters.
- Global illumination: screen-space GI; baked lightmaps and spherical-harmonic
  light volumes (og-bake, `og bake lightmaps`, CPU).
- Physical sky (Rayleigh, Mie, ozone), volumetric and layered clouds, weather
  with wet surfaces, puddles, snow and lightning, a day-night cycle, wind.
- Water: a Gerstner ocean with whitecaps, flow maps, buoyancy (`og.Water`).
- Terrain: geometry clipmaps, up to sixteen layers, holes, scatter, runtime
  edits, octahedral impostors, wind on placed models.
- Decals; particles with mesh, lit and distortion modes, sub-emitters and
  events; trails, beams and ribbons; flipbooks and sprites.
- 2D: sprite batches, 2D lights (point, spot, global), polygon occluders and
  shadow maps, normal-mapped sprites.
- Large worlds: camera-relative rendering and a floating origin
  (`og.WorldOrigin`, in `og.transform`).
- Procedural geometry: three.js shapes, extrude, lathe, sweeps, CSG
  (`og.shapes`; og-geom).
- Video: `type video` clips as texture arrays, or AV1 streams decoded by
  rav1d natively and by WebCodecs (rav1d in a Worker as a fallback) on the web
  (`og.video`; og-video, og-video-wasm).
- Debug draw and debug views (physics, navigation, bounds, LOD, lighting
  terms); development builds only.

**Animation and characters** (`og.anim`, `og.motion`, `og.tween`, `og.path`; og-anim, og-motion, og-tween)
- Skeletal animation from glTF, blend graphs with layered state machines,
  root motion, two-bone and FABRIK IK, blendshapes, 2D skeletons.
- Cloth on skinned garments (`og.Garment`), VAT crowds from baked clips
  (`og.CrowdAnimation`).
- `og.motion`: generated humanoid motion from a model distilled from NVIDIA
  MotionBricks, presentation only, on the CPU or wgpu compute.
- Auto-rig of layered 2D parts into a 2D rig with built-in clips (og-cook).
- Tweens of any reflected numeric field; arc-length splines.

**Physics** (`og.physics`; og-physics)
- rapier3d and rapier2d with enhanced determinism: bodies, compound colliders,
  joints with break force, queries, character and platformer controllers,
  vehicles with a drivetrain, ragdolls, ropes, cloth, soft bodies, 2D and 3D
  particle fluids, buoyancy.

**Navigation and AI** (`og.nav`, `og.ai`; og-nav, og-ai)
- Grid A* and jump point search, flow fields, a tiled navmesh with funnel
  paths, runtime rebuilds, off-mesh links, area costs, RVO crowds, steering.
- Behaviour trees and state machines as `.ogt` data, perception (sight and
  hearing), per-agent blackboards.

**Audio** (`og.audio`; og-audio)
- A deterministic command stream in the tick, mixed outside it (AudioWorklet
  on the web): buses, spatialisation, sound events, QOA streaming; MP3, Ogg
  Vorbis, FLAC and Opus sources through the cook; procedural sound presets
  (`og sfx`).

**UI** (`og.ui`; og-ui, `packages/engine/src/ui`, `packages/design`)
- An immediate-mode toolkit with Clay-style layout, drawn through og-render
  and addressable by the probe; rich text, RTL and CJK shaping.
- UI styles: each game ships its own (`type style`); built-in style packs
  under `templates/styles`, `og style eject`; the default style is generated
  from `packages/design`'s tokens.
- The kit: controls, containers, data views (virtualised lists, tables,
  charts), feedback, HUD and world components, and genre patterns
  (inventory, cards, shop, gacha, lobby, draft, dialogue, quest log, maps,
  garage, chat, RTS commands, photo mode, settings tree and more).
- Screen documents for every screen type of `docs/ui.md` Appendix B, copied
  into a game with `og ui eject`.
- Pad virtual cursor, focus groups, tap, double-tap and long-press
  recognisers, cursor shapes, a player-customisable HUD layout.

**Gameplay systems**
- Dialogue and quests as data, every line an i18n key (`og.story`; og-story).
- Timelines and cinematics: camera, transform, animation, audio, captions,
  property, light, material, post, blendshape, nested and shot tracks
  (`og.timeline`; og-timeline); virtual cameras with priorities, blends and
  collision (`og.camera`).
- Tilemaps: tilesets, auto-tiling, Tiled and LDtk import, runtime edits,
  isometric and hex (`og.tilemap`; og-tilemap).
- Procedural generation: Perlin, simplex, FBM, Poisson disk, BSP, WFC
  (`og.procgen`), on the engine's `og.Rng` (`og.rng`).
- Saves with a version per game and TypeScript migrations; prefabs with
  variants and overrides (`og.world`, `og.load`).
- Input: actions with rebinding and contexts, gamepads with mapping tables
  and rumble, touch, pointer lock (`og.input`; og-input).
- Localization: string tables, an ICU MessageFormat subset, CLDR plurals
  (`og.i18n`; og-i18n).
- Time groups for per-entity time scale and hit stop (`og.time`).

**Worlds and assets** (`og.load`, `og.stream`; og-cook, og-bundle, og-blob)
- The cook: glTF (Draco, meshopt, quantization, KTX2), FBX, OBJ, KTX2/Basis
  textures, MSDF fonts, audio, scenes; incremental, with a slowest-assets
  report.
- Streaming groups, world cells with entities and colliders, sub-scenes,
  texture mip streaming; uncooked glTF and image loads at run time.

**Multiplayer** (`og.net`; og-net)
- Deterministic lockstep and rollback over snapshots, late join, drop and
  rejoin, desync detection with a component diff, a local WebSocket relay
  (`og relay`), quick play through the relay's queue.

**Platform** (`og.platform`)
- The host ABI with game-scoped calls: cloud store, leaderboards, telemetry,
  purchases and entitlements, rewarded and interstitial ads, achievements,
  identity, friends and invites, share. The platform implements them; the
  native host answers from a local JSON store.

**Tooling**
- `og` (below), the probe (`og.probe`; og-probe) and the command channel
  (`og.commands`), the inspector and transform gizmos in development builds.
- Data-parallel lanes inside a system, on rayon natively and Workers on the
  web, with the same results (og-lanes).
- The knowledge package: 24 skills, the generated API wiki, the three-to-og
  guide, the style catalogue (`packages/knowledge`, shipped inside
  `@origingame/engine` as `knowledge/`).
- The showcase archive of frames and numbers (`showcase/`,
  `scripts/showcase.ts`).

The remaining API modules: `og.game` (the game entry), `og.system` and
`og.systems` (systems, and the ones the engine ships in TypeScript),
`og.reflect`, `og.math`, `og.art` (a look's art contract, `docs/art.md`).

## Install

```sh
curl -fsSL https://origingame.ai/downloads/engine/install.sh | sh   # Windows: irm https://origingame.ai/downloads/engine/install.ps1 | iex
og new my-game                            # the latest release; --archetype runner, --premise "one line", --engine <version>
cd my-game && og doctor && og test . && og dev .
```

`og` is a launcher (`packages/launcher`): it runs the engine version a
game's `project.ogt` pins, downloading it once into
`~/.cache/origin-engine` from `https://origingame.ai/downloads/engine/`,
signed with the engine's key (`docs/release.md` §1). A version is one
archive per platform (darwin-arm64, linux-x64, win32-x64) carrying the API,
the runtime, both web hosts, the shell, `og` and the native addon, the
templates and archetypes, and the agents' knowledge. It needs bun; no npm,
no Rust, no checkout. linux-arm64 has no prebuilt `og` yet. Inside this
repository, a project runs against the checkout with
`OG_ENGINE_PACKAGE=packages/engine`.

## Repository layout

```
crates/            Rust workspace
  og-core          handles, reflection, .ogt text, deterministic rng and math, time (og-core-derive: #[derive(Reflect)])
  og-ecs           archetype ECS, schedule, resources, events, snapshot/restore
  og-idl           the IDL model, validator and generators (TypeScript, bindings, schema, wiki, host traits)
  og-api           api/<module>.ogt, the generated host traits, the host-neutral runtime surface
  og-records       the record and column tables generated from the IDL
  og-runtime       the host-neutral implementation of the API
  og-host-web      wasm-bindgen host, run in a Worker on an OffscreenCanvas
  og-host-node     napi host (origin-engine.node), headless
  og-render        wgpu renderer
  og-anim          skeletal animation, blend graphs, IK, blendshapes
  og-motion        generated humanoid motion (CPU and wgpu compute)
  og-tween         tweens of reflected fields
  og-timeline      splines and the timeline sequencer
  og-physics       rapier2d/rapier3d integration
  og-nav           grids, flow fields, navmesh, steering
  og-ai            behaviour trees, state machines, perception
  og-audio         command stream and mixer
  og-ui            immediate-mode UI toolkit
  og-input         the Input resource, events, the shell's input ring
  og-i18n          string tables, MessageFormat, plural rules
  og-story         dialogue and quests
  og-tilemap       tilesets, layered maps, auto-tiling
  og-geom          procedural geometry and CSG
  og-net           room protocol, lockstep, the native relay
  og-lanes         deterministic data-parallel work
  og-video         AV1 decoding natively (rav1d); og-video-wasm: the browser fallback decoder
  og-cook          assets/ to cooked/ .ogb blobs
  og-bundle        cooked blob formats and their runtime loaders
  og-blob          the .ogb container
  og-bake          lightmaps and light volumes (CPU)
  og-probe         command channel and probe over JSON-RPC
  og-cli           the `og` tool
  og-tests         gfx reference harness; og-tests-cooked, og-tests-web: cooked and in-browser variants
packages/
  engine           @origingame/engine: TypeScript API and runtime, web hosts, shell, the UI kit (src/ui); a dist's engine/
  create           @origingame/create-game: the templates `og new` writes; a dist's create/
  design           @origingame/design: the design language and tokens shared with origin-agent; a dist's design/
  knowledge        skills, guides, the generated wiki, looks, scaffold (shipped inside the engine package)
  launcher         og, the launcher: install.sh, install.ps1 and the version a game pins (docs/release.md §1)
templates/         game (3D), game-2d, archetypes/, looks/, styles/, ui-kit, fx-showcase
examples/          art-style samples, sample games and feature samples (autorig, motion-course, uncooked-load, ...)
tests/             evidence/, install/ (the install test), replay/ (replay goldens)
scripts/           check.sh (the gate), lane.sh, worktree.sh, release.sh, wave-end.sh, showcase.ts, ...
tools/             art samples, benches, wasm determinism
third_party/       vendored sources (basis-universal, xatlas, ltc, cleanedge, pixelorama)
docs/              design.md (normative), ui.md, roadmap.md, release.md, ci.md, art.md, ...
showcase/          the showcase archive's index
```

## Commands

`og --help` lists every option; `og <command> --help` shows one.

```
og run              cook, type-check, bundle and run on the native host with the probe;
                    --commands "t:action[:args];..." scripts a run; --device plays the publish build in a browser or on a phone
og dev              watch, rebuild, re-cook and hot-reload through the probe
og build            export the game to dist/; --publish adds the PWA and keeps only player-facing probe members
og cook             assets/ into cooked/, incrementally; --report, --watch
og check            type-check the game code (tsrs, tsgo as a fallback)
og test             the project's bun tests, then each tests/*.og.ts playtest headless
og probe <method>   one probe call to a running og run or og dev
og replay           record a run's per-tick world hash, or compare a trace and report the first divergent tick
og render           render a script or a timeline to frames, a video and a contact sheet
og compare          a captured frame against a reference image (silhouette IoU, ΔE00)
og polish           score the playtests against the taste and feel checks, with a fix per finding
og ui               check, preview or eject screens (per style, breakpoint and locale)
og style            list, eject, upgrade, pack and check UI styles and the art contract
og prefab           check, show and extract prefabs
og i18n extract     the keys dialogues and quests use, against each string table
og sfx              procedural sound presets
og bake             lightmaps, reflection probes, and projection texturing of a level
og migrate three    rewrite a three.js file towards og where the mapping is mechanical
og relay            a local WebSocket relay for rooms
og upgrade          move the engine pin and apply the changelog's codemods
og api              find and show API entries; gen and check the generated files
og doctor           check bun, the engine package, the addon, the wasm, the renderer and the toolchain
```

## Development

`scripts/check.sh quick` (fmt, clippy, drift, the logic tests, the knowledge
and create packages) is the loop while working; `scripts/check.sh` is the full
path-aware lane, `scripts/check.sh functional` what a merge passes, and
`scripts/wave-end.sh` the wave's end measurements (`docs/ci.md` §Lanes).

Read `AGENTS.md` before changing anything: ownership, contracts and the gate
discipline. Toolchain versions are in `docs/ci.md` §Toolchain.
