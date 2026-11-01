# Roadmap to commercial quality

The owner's order of 2026-11-01: every functional gap between this engine
and what commercial games need is closed, not only the open rows of
`docs/release.md` §4. The coordinator (the session that merges into main)
runs this plan to the end. This file is the plan and its status; each
stream's detail lives in the doc it names, and its evidence in its round's
notes.

The gaps were found on 2026-11-01 by reading the code against Unity, Unreal
5, Godot 4 and three.js/Babylon on WebGPU, and against what shipped games of
the common genres need. Each was confirmed in code unless marked
*unconfirmed*; a stream confirms those first and drops what turns out to
exist.

## 1. Rules

- **Order.** The UI wave (`docs/ui.md`, U6 and U8) comes first and is held
  to `docs/ui.md` §15.4. The streams of wave G1 run beside it because they
  own other files. The realism wave (R, `docs/release.md` F10) starts once
  U8 has merged. G2 and G3 fill the slots as they free.
- **Threads.** At most five sub-threads at once, `external-agent:claude-code`
  on Opus 5.5 medium (AGENTS.md); file-splitting threads in Amp `high`.
  Each stream is one thread on one branch from `origin/main`, owns the
  files its row names, and does not edit another running stream's files.
- **Done.** A stream is done when it is merged through the functional lane
  on macrunner with its Metal references read and recorded, it carries the
  tests that fail on the old code, its API is in the IDL (both hosts), its
  wiki and skill text are written, and an archetype or example uses it. A
  stream that changes the web path also passes `web-reference` at its
  wave's end.
- **API.** New members follow `docs/design.md` §3: desc structs with zero
  defaults, errors as values, one API on both hosts (a backend limit is a
  capability query, never a missing member). Breaking changes go in a minor
  version with `changelog.json` entries and an `og upgrade` codemod.
- **Determinism.** Nothing here reads the wall clock inside the tick or lets
  rendering feed back into simulation (render interpolation, dynamic
  resolution and quality scaling are presentation only).

## 2. Waves and streams

Size: S under a day, M one to three days, L a week, XL more (one thread per
stream; an L or XL stream may be split by the coordinator).

### U. The UI wave (first; `docs/ui.md` §11, §17, §18)

| # | Content | Owns | Depends | Size |
|---|---|---|---|---|
| U6a | Foundations and controls: every button variant and hold-to-confirm, toggles, checks, radios, sliders, steppers, selects, segmented tabs, fields, search, colour picker, key-binding capture (on G1-2's action map), on-screen keyboard | `packages/engine/src/ui/controls/`, og-ui widgets it needs | G1-2 for capture | L |
| U6b | Containers, navigation and data: panels, scroll, tabs with bumpers, carousel, accordion, split views, sheets, modals, popovers, menus, radial wheels, breadcrumbs, action bar, pagination; virtualised lists, tables (10k rows), trees, pan-and-zoom graphs, charts, stat-compare rows, calendar | `packages/engine/src/ui/containers/`, `ui/data/` | — | XL |
| U6c | Feedback, HUD and world components: toasts, badges, tooltips, progress, counters with notation, celebrations, subtitles to XAG 104; every HUD row of Appendix C; every world row of §14 | `packages/engine/src/ui/feedback/`, `ui/hud/`, `ui/world/` | — | XL |
| U6d | Genre patterns and screen documents: inventory, paperdoll, card hand, rewards, gacha, shop, battle pass, lobby, select and versus, draft, dialogue, VN box, quest log, maps, song select, garage, ledgers, chat, command cards, HUD editor, photo mode, the settings tree; a screen document for every Appendix B row and `og ui eject` | `packages/engine/src/ui/patterns/`, `ui/screens/`, og-cli `ui` | U6a–c parts it uses | XL |
| U8 | The kit's genre showcases, every seed style on them, archetypes and examples migrated, the fixed templates removed, the seed-sheet notes of U4b (bubbly's loading fill, kawaii's borderless secondary buttons, the light seeds' dim line, inkwash, holo against cyber), the Gallery overlap fixed | templates, examples | U6 | XL, by genre family |

### G1. Every game hits these (beside U6)

| # | Gap | Content | Owns | Size |
|---|---|---|---|---|
| G1-1 | No interpolation between ticks: 60 Hz simulation judders on 120/144 Hz screens | Extraction keeps the previous tick's transforms and the frame blends by the accumulator's fraction (camera, skinned poses, particles' emitters); opt-out per entity for teleports; probe and capture stay on tick state | og-runtime `extract`, og-render instances | M |
| G1-2 | No action map or rebinding; gamepads with non-standard mapping dropped, no deadzones, four pads | Actions with bindings over keys, buttons, axes and touch, contexts, rebinding with conflicts, saved bindings; non-standard pads through a mapping table, deadzones, device kind and glyphs, rumble API, eight pads; release keys on blur (*unconfirmed*) | og-input, `packages/engine/src/web/gamepad.ts`, `web/input.ts` | L |
| G1-3 | Optimised glTF refused, extensions dropped, few texture formats | Draco, `EXT_meshopt_compression`, `KHR_mesh_quantization`, `KHR_texture_basisu`, `EXT_texture_webp` read; `KHR_texture_transform` (runtime UV transform), `KHR_materials_unlit`, `KHR_lights_punctual`, `EXT_mesh_gpu_instancing`, `KHR_materials_variants` honoured; webp, tga, psd (flattened) and dds sources | og-cook importers, og-render material UV transform | M |
| G1-4 | Phones run hot, no recovery from device loss, no GPU memory view | Frame-rate cap and power mode (30/60/native), automatic quality from frame time with hysteresis, render scale and dynamic resolution, device-lost recovery (rebuild GPU resources from the bundle), GPU memory accounting in the memory report and `og doctor` budgets | og-host-web, web shell, og-render `quality`, og-runtime `memory` | L |
| G1-5 | Saves break across versions; whole-world only, capped | A save version per game, migration hooks in TypeScript (rename, transform, drop), engine-component migrations shipped with `og upgrade`, selective saves (tagged entities and resources), the cap measured against a 5,000-entity world and raised or streamed | og-runtime save, og-ecs, og-cli `upgrade` | L |
| G1-7 | PC is a first-class target (owner, 2026-11-01), and its presentation has no settings | A graphics settings model and its page in U6d's settings tree: quality preset, render scale, frame cap and vsync, fullscreen; aspect ratios from 16:10 to 32:9 with HUD safe zones and UI that never stretches; a window moved between monitors of different DPI; mouse sensitivity, raw input and acceleration off; bindings by physical key (`code`) so AZERTY and QWERTZ play, shown by the layout's label | og-host-web, web shell, og-render `quality`, og-ui safe area | M |
| G1-6 | Errors point at bundle lines; crash reports carry a name | Source maps in dev and publish (uploaded beside the build, not served), stacks mapped in `og` and the platform; crash reports with stack, tick, engine version, device, GPU and adapter limits; frame-time and hitch histograms and session events in telemetry | `packages/engine/src/node` build, web shell transport | M |

### G2. Gameplay depth (after G1, before or beside R)

| # | Gap | Content | Owns | Size |
|---|---|---|---|---|
| G2-1 | No prefabs | Prefab assets with nested prefabs, variants, per-instance overrides kept on re-cook, apply back from an instance, `og` and probe support | og-cook scenes, og-runtime spawn, og-ecs | L |
| G2-2 | ECS without reactivity or spatial queries | Change detection, add/remove hooks, run conditions and game states on systems, `children()`; a spatial hash with radius and k-nearest queries without colliders | og-ecs, og-runtime | M |
| G2-3 | Physics: one collider per body, sleep off, first hit only, no breakable joints, arcade vehicles | Compound colliders from children, sleeping with the rest rule kept deterministic, raycast-all and point projection, joint break force and contact hooks, a drivetrain vehicle (gears, RPM, torque curve, handbrake, drift tuning, N wheels), PBD soft volumes | og-physics | XL |
| G2-4 | Navigation and AI | Off-mesh links (jump, ladder, door, teleport), area types and costs, RVO/ORCA crowds, obstacle shapes beyond boxes, several agent profiles, behaviour-tree actions that stay running, subtree references, utility scoring | og-nav, og-ai | L |
| G2-5 | Audio | A bus graph with mixer snapshots, sound events (random, round-robin, pitch and volume jitter, cooldowns), PCM made from TypeScript, EQ, delay and distortion; Opus for music and long sounds beside QOA | og-audio, og-cook audio, og-bundle `qoa` | L |
| G2-6 | Tilemaps, procedural generation, formatting, time | Tilemaps created and resized at run time, isometric and hex orientations; noise (Perlin, simplex, FBM), WFC, Poisson disk, BSP rooms, weighted picks and shuffles on the engine's Rng; compact, date, time and currency formatting; time scale per entity group | og-tilemap, og-core, og-i18n, og-runtime time | L |
| G2-7 | Multiplayer: lockstep only, a stalled peer freezes the room, fixed input shape | Rollback over the deterministic snapshot (GGPO-style, input delay plus prediction), stall timeout, drop and rejoin, custom input schemas (RTS command lists), quick play and matchmaking through the host ABI | og-net, og-runtime snapshot, og-api | XL |
| G2-8 | Genres without archetypes; no dialogue or quests | Archetypes for FPS, racing, card, physics puzzle, RPG and narrative, RTS, visual novel, sports; a dialogue and quest API (graph asset, conditions, variables, localisation) | `templates/archetypes`, a new module | XL, by genre |

### G3. Content, platform and tooling (beside G2)

| # | Gap | Content | Owns | Size |
|---|---|---|---|---|
| G3-1 | No FBX, OBJ or Spine | FBX and OBJ through the cook (skins, animations, materials), Spine JSON and binary into the 2D rig | og-cook importers, og-anim 2D rig | L |
| G3-2 | No real video | Streamed playback: `<video>` and `importExternalTexture` or WebCodecs on the web, a native decoder for executors; audio sync, seek, loop; the texture-array clip stays for short loops | og-render video, og-host-web, og-host-node | L |
| G3-3 | Streaming moves assets, not entities | World cells with entity and collider load and unload, sub-scenes, texture mip streaming | og-runtime stream, og-cook | XL |
| G3-4 | Assets cannot load uncooked at run time | glTF and image loading in the runtime for generated and player-made content, within budgets | og-bundle, og-runtime load | M |
| G3-5 | Every build re-downloads the game | Content-hashed file names and an incremental service-worker update | web build, shell | S |
| G3-6 | No monetisation or identity calls | Host ABI and TypeScript API for purchases and entitlements, rewarded and interstitial ads, achievements, player identity, friends and invites, native share; the platform implements them (origin-agent, origingame) on the ABI's contract | og-api, og-runtime platform, shell | L engine, platform owned beyond |
| G3-7 | No inspector or gizmos for humans | The inspector and transform gizmos `docs/design.md` §1 promises, on the reflected schema, in dev builds, writing back through scene write-back | og-runtime, og-render debug, web shell | L |

### R. The realism wave (after U8; `docs/release.md` F10)

| # | Content | Size |
|---|---|---|
| R1 | Meshlets after three.js#35191: clusters and simplified levels in the cook, per-cluster frustum, normal-cone and two-phase occlusion culling, screen-space-error levels, indirect draws, shadows through the same path; then the visibility buffer with compute rasterisation of small triangles | XL |
| R2 | Temporal upscaling (TAAU) on G1-4's render scale | M |
| R3 | Dynamic GI: screen-space GI plus probes updated at run time | XL |
| R4 | Sky and weather: physical scattering, clouds (layers and volumetric), wet surfaces and puddles, snow build-up, lightning | L |
| R5 | Water: an ocean wave model (FFT or Gerstner) with flow maps | M |
| R6 | Terrain and vegetation: holes, more than four layers, impostors for distant copies, wind on placed trees | L |
| R7 | Decals with normals and no 32 cap; particles with mesh and lit render modes, sub-emitters and events, distortion | L |
| R8 | Shadows and post: more cascades and shadowed local lights, static shadow caching, contact shadows, lens flare, SMAA, HDR output | M |
| R9 | Characters: more than eight blendshapes (64), cloth on skinned garments, GPU animation for crowds (VAT) | L |
| R10 | Cinematics and cameras: timeline tracks for properties, lights, materials, post, visibility, blendshapes and nested timelines, camera roll and focus; camera collision and virtual cameras with priorities and blends | M |
| R11 | 2D lighting: normal-mapped sprites, spot and global lights, polygon occluders | M |
| R12 | Large worlds: camera-relative rendering and a floating origin (*unconfirmed* gap) | M |

Acceptance of R: a photoreal forest reference scene and three.js#35191's
example scenes, frames on Metal, frame times against three.js on WebGPU.

## 3. Compatibility and real devices: last (owner, 2026-11-01)

After R, as the plan's final stage:

- **Reach.** WebGPU only leaves roughly 15–25 % of mobile-web players unable
  to play (iOS below 26, Android below 12, Firefox for Android), an estimate
  from caniuse. The share is measured on the platform's real traffic and
  the owner decides then; until then the WebGL2 tier stays retired.
- **Real devices, PC and phone alike.** PC is measured as fully as phones:
  Windows with NVIDIA, AMD and Intel integrated GPUs, a Mac on Apple
  silicon, Linux with a Vulkan GPU, each in Chrome, Edge, Firefox and
  Safari where they run, at 1080p, 1440p, 4K and ultrawide. The device list (`docs/release.md` §6 item 5) and every
  real-phone measurement: G1-4's power mode, automatic quality and device
  loss are built and tested on the runners until then, and their phone
  numbers are taken here.

## 4. Status

| Stream | Thread | Branch | State |
|---|---|---|---|
| G1-2 | [T-d77ec7d3](https://ampcode.com/threads/T-d77ec7d3-271f-4b67-9d15-d46a5959df5a) | `g1-input-actions` | started 2026-11-01 |
| G1-6 | [T-01a197e9](https://ampcode.com/threads/T-01a197e9-a4d5-742d-9c41-fa7086e21b35) | `g1-sourcemaps-telemetry` | reviewed at 84739b70, merging with browser checks |
| U6b | [T-90964ef0](https://ampcode.com/threads/T-90964ef0-8bb4-40fd-947e-90c23eeb1649) | `u6b-containers-data` | started 2026-11-01 |
| U6c | [T-52f25cda](https://ampcode.com/threads/T-52f25cda-5a9c-42e6-a2b3-dbe62c284b35) | `u6c-feedback-hud-world` | started 2026-11-01 |
| split-runtime-3 | [T-76cca373](https://ampcode.com/threads/T-76cca373-5286-44fd-849d-775944160cf5) | `split-runtime-3` | reviewed at 0626fc39, merging |
| G1-1 | [T-ed474375](https://ampcode.com/threads/T-ed474375-d637-48c0-8d9a-9ca91cf6b7b6) | `g1-render-interpolation` | started 2026-11-01 |
| G1-5 | — | — | next free slot |
| og-cook cook/mod.rs, look.rs split | — | — | after G1-3 |
| G1-3 | [T-01a1980c](https://ampcode.com/threads/T-01a1980c-a132-73a9-96bf-c2ebfbde9c2e) | `g1-gltf-formats` | started 2026-11-01 |
| G1-4, U6a, U6d | — | — | as slots free (U6a after G1-2) |
