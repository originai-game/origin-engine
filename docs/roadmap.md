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
  U8 has merged; R1, R2, R4, R5 and R8 started early beside wave B to use
  the eight slots (the owner, 2026-11-02). G2 and G3 fill the slots as they free.
- **Threads.** At most eight sub-threads at once, `external-agent:claude-code`
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
| G3-1 | No FBX or OBJ; 2D characters need hand rigging | FBX and OBJ through the cook (skins, animations, materials); layered PSD or PNG parts auto-rigged into the 2D rig with built-in clips (the owner, 2026-11-02: no Spine) | og-cook importers, og-anim 2D rig | L |
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

After every feature is done, not in parallel with optimisation (the
owner, 2026-11-04): compatibility starts only once the feature work,
R included, is complete.


- **HDR output on a real display (owner, 2026-11-04).** R8's HDR output
  (feat/hdr-output) lands opt-in with its SDR frames unmoved; its
  acceptance on a real HDR display (the og-tests `surface` driver on an
  unlocked, visible XDR desktop, then the art director) runs here. Until
  then HDR is not called accepted.

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

**Wave B (the owner, 2026-11-02: speed first; macrunner's Metal stays the only pixel judge, authors never touch a GPU; wave A stopped with its work pushed and its findings carried into these briefs):**

| Thread | Owns |
|---|---|
| [T-e1749dc1](https://ampcode.com/threads/T-e1749dc1-22c5-4c30-8209-33772ef1dc1f) | world: G3-3 batches 2–3, R12 batches 2–4: all done 2026-11-02 (G3-3 b2 and R12 landed; G3-3 b3 mip streaming with macrunner) |
| [T-01a19d41-2bc3](https://ampcode.com/threads/T-01a19d41-2bc3-71a3-9025-50ed798cfe89) | og-ui: ui-gaps B2–B7, then the selection grammar, slider track, art notes and the unpainted style packs |
| [T-01a19d41-3600](https://ampcode.com/threads/T-01a19d41-3600-743b-bb6c-5361bfee4fb2) | tools: G3-7 batches 1–3, R10 batch 3 (shots), G3-1 batch 5: all three done 2026-11-02, with macrunner |
| [T-01a19d41-442d](https://ampcode.com/threads/T-01a19d41-442d-7563-bb21-0b7907d172d2) | G2-7 batch 6, R11 batches 2–4: G2-7 done on main; R11 done 2026-11-02, with macrunner (r11-2d-lighting-4) |
| [T-01a19d41-4c76](https://ampcode.com/threads/T-01a19d41-4c76-71d9-a16b-ef8bb985a436) | UI content: RTS and sports HUD, the holo kit screens, U6d's remaining patterns, screen documents, `og ui eject`, the settings tree: done 2026-11-02, last head u6d-r42 with macrunner |
| [T-a558a897](https://ampcode.com/threads/T-a558a897-87a0-4c1d-82c8-f218e5e0e18b) | render: R1 meshlets, culling, the visibility buffer |
| [T-01a19d59-9055](https://ampcode.com/threads/T-01a19d59-9055-707b-8b4e-f2a9a91c4030) | render: R2 TAAU, R8 shadows and post; environment maps: background blurriness, a ground-projected skybox, blending two maps, runtime HDR loading (the owner, 2026-11-02) |
| [T-01a19d59-9d85](https://ampcode.com/threads/T-01a19d59-9d85-7604-ba45-b447cc453814) | render: R4 sky and weather, R5 water: both landed 2026-11-03 (R4 4076017e8 and its shelter and art rounds; R5 main 748fab859); the game-3d replay golden needs `--update` at release |
| [T-c113ecea](https://ampcode.com/threads/T-c113ecea-ad88-4d45-83d2-97ed7f2b6561) | render: R7 decals and particles, R9 characters (blendshapes, cloth, VAT crowds): all landed 2026-11-02 (VAT crowds last, main 6c90534f0); crowds are CPU-culled, not yet GPU-culled or on the meshlet path |
| [T-e522b299](https://ampcode.com/threads/T-e522b299-b8c9-44cb-8bf4-ff01169395c1) | render: R6 terrain holes, layers, impostors, wind: all landed 2026-11-02 (main 09c1b035c); limits: impostors take no shadow of their own, a view from below draws the horizon frame, cards don't sway |
| [T-aa5a5083](https://ampcode.com/threads/T-aa5a5083-c49e-4586-9621-ca6f7e8dec57) | render: R3 dynamic GI (screen-space GI, run-time probes): complete 2026-11-03 (batch 1 SSGI 4dd044b5f; batches 2–3 og.RealtimeLightVolume and probe specular, main 6c0bd7490); functional milestone pending |
| [T-d800cb02](https://ampcode.com/threads/T-d800cb02-5ecd-48bb-ba24-b72b7f152853) | UI: U8, the kit's genre showcases, every seed style on them, examples migrated, fixed templates removed (the owner, 2026-11-02) |

**Paused (the owner, 2026-11-03, holding the cap at 8):** the introduction site, [T-01a1a06d](https://ampcode.com/threads/T-01a1a06d-d946-751e-9eb1-fb73ee1a3538), branch `site` at f9302d39a, pushed and not deployed; it deploys only once the features it shows are done. The owner (2026-11-04): the site is to be re-planned before any recording or deploy, because some of what it would show still looks too rough; Skyreach Meadow's art-approved close shots (site-scenes c23a4a3be) are kept, but its cinematic footage is not recorded until that plan exists. A fresh thread resumes it once these arrive: site-hero's scene build (then confirm `game.custom.hero.*` from a real publish build), studio-win's per-scene perf and the cold first load on the real host, site-motion in `public/live/motion`, U8's accepted UI showcases in place of the interim og-ui frames, final media for the 45 `check.ts --final` blockers, a stronger characters poster, and the UI grid controls' spacing under the nav.

Deferred: UI text scale at 200 % (the owner, 2026-11-04: the bar, the kit cells and the player setting stop at 150 % for now; 200 % returns as its own batch), the overlay's GPU cost (perf phase), accessibility, the genre archetypes, the platform asks' joint review, compatibility (DX12, browsers, phones).
The rows below are the previous threads, kept for their history.


| Stream | Thread | Branch | State |
|---|---|---|---|
| G1-2 | [T-d77ec7d3](https://ampcode.com/threads/T-d77ec7d3-271f-4b67-9d15-d46a5959df5a) | `g1-input-actions` | merged 2026-11-01 (main 1fa0b316; follow-up g1-save-empty-types merged at 1c1451ea) |
| G1-6 | [T-01a197e9](https://ampcode.com/threads/T-01a197e9-a4d5-742d-9c41-fa7086e21b35) | `g1-sourcemaps-telemetry` | merged 2026-11-01 (main 3aca1209); Chrome checked on macrunner; Chrome and Edge passed on studio-win (RTX 5060, gpu names nvidia/blackwell); Safari blocked (macrunner's GUI session locked) |
| U6b | [T-90964ef0](https://ampcode.com/threads/T-90964ef0-8bb4-40fd-947e-90c23eeb1649) | `u6b-containers-data` | merged 2026-11-02 (main d60ee998, 107 components references); its thread continues with ui-scale |
| U6c | [T-52f25cda](https://ampcode.com/threads/T-52f25cda-5a9c-42e6-a2b3-dbe62c284b35) | `u6c-feedback-hud-world` | ability bar, unit, cast, party and raid frames merged 2026-11-02 (main e3ec45ab through round 17; name outline and drop from round 18 landing); next: racing, RTS and sports HUD batches |
| split-runtime-3 | [T-76cca373](https://ampcode.com/threads/T-76cca373-5286-44fd-849d-775944160cf5) | `split-runtime-3` | merged 2026-11-01 (main 3aca1209) |
| G1-1 | [T-ed474375](https://ampcode.com/threads/T-ed474375-d637-48c0-8d9a-9ca91cf6b7b6) | `g1-render-interpolation` | merged 2026-11-01 (main 3ad53786, with g1-save-added-fields); 120 Hz look waits on macrunner's console |
| G1-5 | [T-4157fcd5](https://ampcode.com/threads/T-4157fcd5-17e7-43b7-8471-59ff1b6b6356) | `g1-save-migration` | merged 2026-11-01 (main 1b4c54f6, with g1-save-names and the typecheck fixes); cap stays 512 KiB (5,000 entities measured 437 KiB); 0.7.7 save restores the same on native and web; replays unmoved; open: split hash differs by one empty archetype, a 0.7.7 restore replays on-screen entrances |
| og-cook cook/mod.rs, look.rs split | [T-01a199ff-1ddf](https://ampcode.com/threads/T-01a199ff-1ddf-75f6-a775-7ac6025fa905) | `split-og-cook` | merged 2026-11-02 (main 63ec3295, split-og-cook-2, verbatim moves reviewed) |
| G1-3 | [T-01a1980c](https://ampcode.com/threads/T-01a1980c-a132-73a9-96bf-c2ebfbde9c2e) | `g1-gltf-formats` | merged 2026-11-01 (main 1c1451ea, five gltf references) |
| G1-4 | [T-532b590f](https://ampcode.com/threads/T-532b590f-cff8-4ccc-89da-b4c97a97180a) | `g1-frame-quality` | merged 2026-11-01 (main 8c6dd264); Chrome and Edge on D3D12 pass frame cap, render scale and device loss; auto quality cycles one rung on the GPU-time path and telemetry lacks deviceLosses, both moved to G1-7 |
| U6a | [T-820be2e3](https://ampcode.com/threads/T-820be2e3-deeb-441a-b49f-aa37ad4dc058) | `u6a-controls` | merged 2026-11-02 (main 69875aa3, kit-controls references, RTL clip and ring-order fixes); its thread fixes the four kit playtests U6b broke |
| G1-7 | [T-29a2d81f](https://ampcode.com/threads/T-29a2d81f-1300-4ea8-b96f-6b9e0a17f76a) | `g1-pc-presentation` | merged 2026-11-01 (main 86ad9f02); follow-up merged 2026-11-02 (main fa1924a5); studio-win recheck passed in Chrome and Edge (gate lock, kit Graphics tab, topdown at DPR 2); done |
| U6d | [T-f9b428a3](https://ampcode.com/threads/T-f9b428a3-912b-4638-884a-40c7035c9ab5) | `u6d-patterns-screens` | pattern screens merged 2026-11-02 (main 8e2ea77f: inventory, gear, cards, rewards, gacha with reveal and summary, combat, saga map, star bar, idle generators, five locales; 170 kit-patterns references at milestone 2); next: store, battle pass, the remaining patterns, screen documents with eject, the settings tree |
| png-references | [T-858f4e1e](https://ampcode.com/threads/T-858f4e1e-a6d7-464a-aaef-a8e3b74d89c0) | `png-references` | merged 2026-11-01 (main 90ee1b43; 423 files pixel-exact, 211.0 → 114.3 MB): record paths write Best + adaptive PNG (fast encoders since 10-17/10-25 made 309 references 167 MB), existing ones recompressed pixel-exact |
| web-tests | [T-01a199ff-9c29](https://ampcode.com/threads/T-01a199ff-9c29-7575-b570-436fa5b42fbe) | `web-tests` | merged 2026-11-02 (main 69875aa3): tick-driven waits, PWA install prompt caught early, "Install <game> to play offline" |
| G2-2 | [T-01a199ff-a49a](https://ampcode.com/threads/T-01a199ff-a49a-70e2-81db-561a827edcc0) | `g2-ecs-reactive` | merged (67a07d57); G2-1 prefabs (df4042f2) and G2-4 (72c5b1eb) complete; G2-7 netcode batches 1–3 merged 2026-11-02 (main c7380cd6: rollback over the snapshot ring with og.Lockstep `mode`, redundant inputs, relay latency and loss injection, stall timeout, drop and rejoin); next: custom input shapes and command lists, quick play, the skill and the cross-host test |
| G2-5 | [T-01a199ff-ac22](https://ampcode.com/threads/T-01a199ff-ac22-70bb-a2ad-e6a2d94f16eb) | `g2-audio` | merged (67a07d57); G2-3, G2-6 and G2-8 og.story complete; the genre archetypes are deferred (the owner, 2026-11-02: features first); R10 batch 1 merged (main 00606971: timeline tracks, camera roll and focus), batch 2 with macrunner (e2602e79: virtual cameras, CameraBrain with blends, solo and collision, all lazy); next: the shot track and dialogue shots |
| ui-glass-render | [T-baf465c8](https://ampcode.com/threads/T-baf465c8-8dd8-4794-9358-e27a9dc4e0ca) | `ui-glass-render` | merged 2026-11-02 (main 513654d8, five ui-glass references recorded) |
| ui-scale | [T-90964ef0](https://ampcode.com/threads/T-90964ef0-8bb4-40fd-947e-90c23eeb1649) | `ui-scale` | merged (4fc95954, neutral default 58bd2eee); og-ui follow-ups landing (gradient mask, undressed parts, 3:1 contrast, text coverage); now G3-7 inspector and gizmos on `g3-7-inspector` |
| G3-1 | [T-f7ce710e](https://ampcode.com/threads/T-f7ce710e-f14d-4634-b416-883d7772c9a7) | `g3-1-importers` | FBX and OBJ merged 2026-11-02 (main 650bb319); Spine dropped (the owner, 2026-11-02); layered-image auto-rig: batches 1–3 merged (humanoid-side skeleton, meshes and weights, ten clips), batch 4 with macrunner (fba6881b: humanoid-front, quadruped and creature templates with ten clips each, rigid and fold tests over every template); next: the docs/art.md contract, rigParts, the mage's run fix |
| G3-2 | [T-01a19c1b-8531](https://ampcode.com/threads/T-01a19c1b-8531-71e4-b29c-0f27f3213805) | `g3-2-video` | merged 2026-11-02 (main 51a03a45, all batches to 5ab42a58: AV1 cook, native rav1d, WebCodecs plus a lazy rav1d-wasm 720p fallback, f64 time, lazy og.VideoPlayer with a save migration); open: browser checks at wave end; its thread now does R11 2D lighting on `r11-2d-lighting`: batch 1 with macrunner (14e6386d: spot and global lights, light layers, lazy records with a migration; four raw hash pins move, no frames); next: polygon occluders, polar shadow maps, light2dAt |
| G3-3 | [T-01a19c1b-8db5](https://ampcode.com/threads/T-01a19c1b-8db5-717c-8040-d074363360a5) | `g3-3-streaming` | batch 1 merged 2026-11-02 (main fab10c7e: sub-scenes and cells, fixed spawn order); next: persist and saves, cell asset groups, the hold gate, the spawn budget, baked colliders; then mip streaming |
| G3-4, G3-5 | [T-01a19c1b-9730](https://ampcode.com/threads/T-01a19c1b-9730-7728-8b82-77cc62cac942) | `g3-4-5-load-update` | complete 2026-11-02: G3-5 landed (63c5fcae, aliases kept until platform-asks #18); G3-4 batches 2–4 with macrunner (head baa07a41: imageData, gltfData, budgets, journaled completions, og.load.release, dump_memory, examples/uncooked-load; +26.9 KB brotli wasm); open: the browser run at wave end; next on this thread: R12 large worlds on `r12-large-worlds` |
| G3-6 | [T-01a19c1b-9fa1](https://ampcode.com/threads/T-01a19c1b-9fa1-7638-a1f3-2ce2e3794614) | `g3-6-platform-abi` | engine side merged 2026-11-02 (main b56c2205, milestone 4 at 21443311); the platform side of platform-asks #18 and #20 (OG_RENDERER=none in the sandboxes) waits until the engine work is done and is reviewed with the owner then, since neither the agent nor the engine is live yet (the owner, 2026-11-02); open: the browser Player check at wave end |
| og.motion | [T-cd152924](https://ampcode.com/threads/T-cd152924-03dd-42d3-a4dc-b5ef6716af67) | `og-motion-runtime` | merged 2026-11-02 (main bf39e907, 6 patches; weights in the showcase bucket, fetched by sha; M4 Pro 677 GFLOPS measured); decode spread and early weight load fixed (8b7c6acde); upright idle (og-motion-train 23be7e5) and the performance API (walkTo braking and heading, lookAt, react) at macrunner on `motion-performance` 5fb8a5c79, [T-01a1a1b8](https://ampcode.com/threads/T-01a1a1b8-4cdd-709a-b6dd-8a717f985239); open: the jump's pitch (clamped, not yet fixed at the source), the wave goal still idle at 12 s, the brace/hood clip (scenes thread, waiting on the gateway's Meshy text-to-motion) |
| ui-gaps | [T-32098740](https://ampcode.com/threads/T-32098740-332f-422b-b4af-3270bb28265c) | `ui-gaps` | started 2026-11-02 (og-ui engine gaps: pad virtual cursor, focus overrides, tap/double-tap/long-press, cursor shapes, customisable HUD and touch layout; auto-fit, inline images and device glyphs, text styles, general typewriter, text on a path; screen transitions that capture the outgoing screen; animated images; split-screen UI, UI profiler, parallax, card tilt, damage merging, beat clock); accessibility deferred (the owner, 2026-11-02) |
