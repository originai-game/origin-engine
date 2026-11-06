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
- **UI is judged by its look (the owner, 2026-11-04).** Contrast is not
  checked for now: `og ui check` contrast findings hold nothing and are
  not fix-forward items. UI lands on quick plus the art director's
  aesthetic review against commercial games (material, hierarchy, real
  art, the pack's identity, selection, typography, the scene behind the
  HUD), tracked in its ranked gap list. The engine owns the baseline:
  components, patterns, screen documents, each pack's design and the
  typography, judged as sheets on the pack's own ground. Fitting UI to a
  particular game's scene is the game agent's work while it builds that
  game; showcase and in-scene frames are demos that hold a landing only
  when something breaks, never on looks (the owner, 2026-11-04).
- **UI is a core plus extensions (the owner, 2026-11-04).** The core holds
  the element model, layout, motion, render, the document and style
  formats and the common components; genre kits and style packs are
  extensions (a directory and a manifest, shipped with the engine)
  installed into a game, built for agents to find, change and create.
  First wave: RPG, shooter, puzzle/casual, gacha, strategy/TD and racing
  kits; fantasy, scifi, cartoon and pixel packs; the other packs frozen.
  The UI runner on macrunner is also the UI art director and merges what
  it approves. Plan: artifact 3d9c6d8e-33f4-4cf3-8e61-54239077cd13.
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
  Reconfirmed 2026-11-05 (the owner): WebGPU only, with a to-do left open
  to study a fallback. The study measures the share without WebGPU on
  real traffic and costs the options: three.js's route (`WebGPURenderer`
  falling back to a WebGL 2 backend, its TSL materials emitting GLSL) and
  what has no WebGL 2 form here (compute, indirect draws, storage
  buffers: GPU culling, OIT, SSSR, GPU crowds, the visibility buffer).
- **Real devices, PC and phone alike.** PC is measured as fully as phones:
  Windows with NVIDIA, AMD and Intel integrated GPUs, a Mac on Apple
  silicon, Linux with a Vulkan GPU, each in Chrome, Edge, Firefox and
  Safari where they run, at 1080p, 1440p, 4K and ultrawide. The device list (`docs/release.md` §6 item 5) and every
  real-phone measurement: G1-4's power mode, automatic quality and device
  loss are built and tested on the runners until then, and their phone
  numbers are taken here.

## 4. Status

**2026-11-05 (the coordinator, from each thread's last report).** Every
wave of §2 has landed apart from what is listed here. The feature areas
of the three audits (2026-11-04) landed on main: GI area 1 (DDGI cascades
ahead of the camera, screen GI across the screen, `both` no longer double
counting), atmosphere and clouds, terrain v2 with the first-frame corner
cache, grass, water area 5 (W7 deferred), stylisation areas 6 and 7,
shadows and the midpoint local shadow maps, weather on surfaces, SSSR
with the min-max Hi-Z march, FSR 3.1, GPU-culled crowds, destruction
M1–M9 (GPU XPBD rigid debris), PR #8's fork077 sections (s42, s55, s14,
s18, s36 with COOKER r11, s24+s72), `pipelines auto`, and Windows native
compatibility. Threads that own each open item are named; hard problems
run on Claude Fable 5.1 (AGENTS.md).

**Release order (the owner, 2026-11-06): 0.8.0 ships only once this phase's UI work is complete** (B0–B9 landed and B5's art review passed), with wave-end #2's failures fixed and a third wave-end green on that head.

**Wave-end #2 at 30b2b996f (2026-11-06): not ready for 0.8.0** ([T-01a1ac98](https://ampcode.com/threads/T-01a1ac98-6bb5-70cb-96ae-1c26a60f2af0); evidence in the measure worktree, `target/evidence/wave-end2-30b2b996f/`). Pass: gate (2753 native, 692 Bun), sizes (3D 4,514,792 B Brotli), cross-target determinism, destruction.test.ts, conformance, release pack, ui-styles. Fail: perf (8 known overruns: gltf-reference 2.378/1.385, lights-256 .937/.546, materials-physical 1.044/.828, showcase 6.004/3.245, showcase-capture 11.170/8.108, spheres-20k-moving 8.084/5.276, sprites-20k 1.851/1.051, raycast-record-20k .383/.320; lights-2d-64 has no baseline); web (fluids3d's WebGPU reference stale after 32e2a7ab5; Meridian 10/10, worse than wave 1, af9bed852..30b2b996f; six fx looks newly failing: gothic-contour, detailed-doodle, black-figure-pottery, heavy-ink-shadows, anime-cel-3d, inked-comic-3d; known: stylise-print, hemisphere-light, Ember title/beacon, pencil-hatching, decorative-storybook, paper-craft, minimal-geometric); art 5/37 over (known); ui-kit 3/44 (B1's three); release verify (step 9's "fantasy cooked shared, storybook" assumption; the container run has no GPU and no `OG_RENDERER=none`); the yard's tick 427 hashes 7bb9d5d5e0a674ea in Chrome and natively, not the golden b81e819120564fb1 (resolved on `wave-end2-fixes`: the golden was wrong, see the wrap-up's line below).

**Wave-end #2's fixes landed at 19b15a1cc** (2026-11-06): the browser looks' failures were the web probe's `frame` feeding automatic quality (a8a92df95: scripted frames render at max scale and stay out of its samples; Chrome then matched all 37 looks twice, byte-identical); Meridian's skyline rim is the towers' own emissive bars after 5ccff020f (704f25f91, 14 frames); Ember Vale's title re-recorded after ac5ea0028; fluids3d's stale WebGPU reference deleted; the yard's tick-427 hash 7bb9d5d5e0a674ea is right on every committed tree (the old golden came from a stale build); `release.sh verify` passes on host and container (f0783bcfb). To-do: gfx stylise-print (5.7 %, hatch phase) and hemisphere-light (RMSE 0.0108) differ deterministically between Chrome and Metal (compatibility phase); Meridian's `screens` layout pins after B0/B7 (with B1); win32-x64 archive on studio-win; perf.

**Next phase: creation choices and the UI foundation (the owner, 2026-11-06).** Plan: artifact febbaf97-4090-406b-96bf-c2e4e8b5c364; research behind it: artifact 04312e7c-59b0-48bd-a8d1-ed2cb7a15d43 (no existing Rust UI library or game-UI middleware meets the engine's constraints, gpui included; og-ui stays, its text stack moves to parley/harfrust/skrifa and the native host gains AccessKit). A user creating a game chooses genre, view, art style (with a mood), UI style, platform and input, and players, with languages and music style defaulted. UI polish must not need a Rust build. B5's art review of the four maintained packs and every kit is the phase's acceptance gate; B6 refuses unrestyled built-in UI in a publish build. Started on macrunner in Amp `high` threads: B0 polish without Rust [T-01a1ac9d-9ef4](https://ampcode.com/threads/T-01a1ac9d-9ef4-743b-aedd-ae4320a3aa59), B7/B8 text stack and AccessKit [T-01a1ac9d-a730](https://ampcode.com/threads/T-01a1ac9d-a730-713e-a0c5-974d7cd3ccd6), creation choices A1/A5–A7/A10 [T-01a1ac9d-b843](https://ampcode.com/threads/T-01a1ac9d-b843-76f5-8537-cc6ecd2b5576), catalogue, `og ui find` and fallback icons B2/B3 [T-01a1ac9d-c222](https://ampcode.com/threads/T-01a1ac9d-c222-756b-a209-418339f4f819). Creation choices A1/A5–A7/A10 landed at 15f9e425b (schema in packages/create/bin/choices.ts, project.ogt's `create` block, platform ask 24); the catalogue, `og ui find` and fallback icons (`ui-catalogue` f9970f1e8) are with the art runner. Started 2026-11-06 in Claude Code: A2/A3 genre starters and views [T-01a1acd3-2e20](https://ampcode.com/threads/T-01a1acd3-2e20-7527-95fc-78af99546f92), A4 look catalogue and mood [T-01a1acd3-3824](https://ampcode.com/threads/T-01a1acd3-3824-73c1-a3d8-6fe731f1548a), A8/A9 languages and audioBrief [T-01a1acd3-3ffc](https://ampcode.com/threads/T-01a1acd3-3ffc-756f-85cf-81b5b0a0a8a5), B1 kit wrap-up [T-01a1acd3-46e5](https://ampcode.com/threads/T-01a1acd3-46e5-72d2-b56a-0e9303c777d2), B6 the game's own UI [T-01a1acd3-50aa](https://ampcode.com/threads/T-01a1acd3-50aa-7023-991d-873409c5a589). Landed by 3a273183c: B0 polish without Rust (to efbb4824a), B8 AccessKit (ask 25), languages and audioBrief (0ac757101, ask 26). B7 text stack landed at 2eef681df; bounced then fixed: ui-catalogue (thumbnail render), ui-own-look waits for it. B9 art pipeline and motion landed at 07ba950fc (painted art recoloured at cook, materials kit, motion timing table, mockup step, layer stack after CommonUI: a changelog break, Back goes to the active layer). Open on main, not from B9: ui-hud's unitFrame/partyFrames and speedometer bun tests fail, and no-renderer's og test timeout is fixed at 513fb9bfb (find_overlaps once per element; og ui check's default screens leave out inspector screenshots; test overhead, clean-up and an orb lane for the non-GPU steps continue in [T-01a1af10](https://ampcode.com/threads/T-01a1af10-012d-76a3-8779-a39f4321971e)). B1 kit wrap-up landed at aed84884b (Meridian `screens` 11/11 again, the tabular-figure regression fixed, the kit frames re-recorded after B7); its author now owns ui-hud's unitFrame/partyFrames and speedometer failures (DrawCmd::Text keeps the source text beside locl glyphs), the fit-label text-scale rule and the pseudo rewards layout. The idle, tower-defense and roguelike archetype reference drift is A2/A3's; platform contracts (2D overlay, choices schema) [T-01a1ae52](https://ampcode.com/threads/T-01a1ae52-3d6f-7673-a257-fe2d76ad9708). Waiting: B4 genre kits (after the catalogue lands; widened by the owner, 2026-11-06: og.ui's core keeps only generic UI, and every genre pattern, genre HUD and genre screen document moves out to an `extensions/<genre>` kit, a changelog break), then B5's art review as the gate. Second wave-end: [T-01a1ac98](https://ampcode.com/threads/T-01a1ac98-6bb5-70cb-96ae-1c26a60f2af0).

**Not implemented yet** (visual simulation closed on 2026-11-05: cloth, soft body and garment at b2f51ae70, GPU MLS-MPM fluid at 32e2a7ab5, GPU particles colliding with scene colliders at 1b63c4c9e; the fluid's look notes and 6.2 ms cost are deferred)

| Item | State | Thread |
|---|---|---|
| GPU scatter placement | on main at 5d083d036: `og.TerrainScatter.gpuPlacement`, the default since the owner's decision of 2026-11-05, compute placement per chunk with HZD-style layered footprints, pixel-identical to the CPU; render CPU 60.3 → 14.3 ms a frame on a moving camera, GPU placement 0.21 ms. The far-edge CPU fallback landed at 0e3a7882e. Open: release-build numbers at the wave's end | [T-01a1aaa2-f809](https://ampcode.com/threads/T-01a1aaa2-f809-707d-a8ee-0d482177372a) |
| UI 1B M2 | ui-tight landed at 958a5340b; `ui-quality` landed (results tabs, focus rings and selected rims over their content, the pseudo-locale keeping rich-text tags whole, every kit frame over the pack's plain ground, the cross-product style sheets retired; the art runner recorded the rest of Metal and the missing text150 cells). To-do: genres2-phone "fire's budget at 60 % is in its row"; settings-saved "the Graphics page holds the display's entries" (`dump_ui` text is "Quality preset", the check wants "QUALITY PRESET"); touch's `move_to_text '=store'` finds no target after `scroll:0,3356` (29 UI elements), so the store/Buy checks fail. Both fail alike with main's test files, not with the plain ground. The deflected stick (`ui-quality-wip` 14ebd96f7: record stick-held.png, check widget-set.png); art notes and the ja logo fit (ec617013a); locale fixes: rewards truncation, the controls Quality ellipsis, the carousel banner title, pause "Next", then ar's RTL mirroring | — |
| HDR output accepted on a real HDR display | landed opt-in; acceptance in §3 | — |
| W7, a deterministic CPU FFT band for buoyancy | deferred | — |
| wgpu 31 | upstream unpublished (`docs/release.md` F5) | — |

**Implemented, below target** (Destruction closed on 2026-11-05 at de54293ff: closed primitive seams, rubble chips in each material, brick joints shedding clay and mortar grit, lit mesh particles blending alpha by default (a CHANGELOG break); its native/wasm divergence was traced to parry's inertia diagonalisation (glamx's closed-form `SymmetricEigen3` calls the host's `acos`/`cos`) and fixed on `destruction-determinism` 81ba76342, with the art runner: glamx vendored under `third_party/glamx` with an f64 cyclic Jacobi solver of `+ - * / sqrt`, bit-pinned; tick 240 hashes e301a0b073a4050e on node and web; `tools/wasm-determinism/run.sh` agrees and Chrome and native agree at tick 427, but it is not landed (the wall test below fails). neon-vector closed on 2026-11-05: the outline owns silhouette edges at 481203728, and the wireframe takes the surface's colour or `wireframeColor` at 5ccff020f. PowerVR's 1:1 linear-sample error was fixed with `sample_exact` for post passes at 2a5ac7165, Metal 481/481 unmoved, with a webgpu-android smaa reference. SSSR met its 2 ms target on 2026-11-05, b4d1c9506: half-resolution denoiser and a joint bilateral upsample, rough metal 3.85 → 1.60 ms, wet street 3.62 → 1.52 ms, mirror 2.57 → 0.97 ms; `ssrResolution` defaults to `half`; its D3D12 race (the upsample wrote the `lit` texture the lit pass reads) was fixed at 8ae87efac, DX12 and Vulkan reruns byte-identical, Chrome 3/3 against Metal, with ssr-wet-floor in the shared browser registry at 226f11fbf)

| Item | Now | Target or problem | Thread |
|---|---|---|---|
| Local shadow maps, point-spot-lights | landed at 3a829d542..342806673 (one-pass faces, rim fix, pass timestamps, Bend's contact shadows as compute with atomicMax masks so the nearest line wins in any GPU order): release 720p 5.346 → 3.944 ms (prepass 0.07, contact 1.55, main 2.30 of which the 8×8 `local_wide` filter 1.30, tonemap 0.05); 1080p 8.25 ms; new baseline 3.944 ms recorded at 342806673 (the old 0.452 predates the midpoint maps and contact shadows; the owner allowed a new baseline if every cost is an intended feature) | further cuts change the look (fewer contact samples, a narrower local filter, half-resolution contact); open: the ball rim's lower-left tip (`local_depth`'s far-side fallback under the 8×8 taps), pcss-moments 3096460c5 | — |
| FSR 3.1 | `upscalers` da0a57836 (loops unrolled for Metal, accumulate's taps shared through a workgroup tile): extra over TAAU at 1080p +0.23–1.87 ms (main +1.41–3.22), at 1440p +0.82–1.25 ms (main +3.26–5.67); the resolve pass is still about twice TAAU's | TAAU's cost, then the default upscaler; next accumulate's 16 history loads | [T-01a1a835-8371](https://ampcode.com/threads/T-01a1a835-8371-7519-a4b0-6d33cc38f6a4) |
| GI probes | main pass 3.70 ms (1.48 without GI); `both` frame 12.70 ms | screen GI reaches 0.65 of the path-traced reference (ceiling 0.72–0.80, the rest off screen); probes 0.81 at 32 frames | area 1 closed; next optimisation when the product needs it |
| First frame | terrain 7–12 s on DX12 after the corner cache; the Pixel's first frame 14–17 s | the shader compile budget; measured at the wave's end and in §3 | web boot [T-01a1a707](https://ampcode.com/threads/T-01a1a707-5055-70bb-ae28-ae6fad4e0388) |
| Wasm size: the text stack's line-break dictionaries | +2.08 MB Brotli with B7 (publish 6.87 MB, gate 16 MiB); turning ICU4X's word dictionaries off saves 1.33 MB but loses Thai, Lao, Khmer and Myanmar breaking, so it is not done | to-do for the optimisation phase (the owner, 2026-11-06: features and look first): pack the dictionaries as data only into games whose languages or cooked fonts need those scripts (ICU4X takes data at run time; check parley 0.12's hook), with a native/wasm text-hash test; attribute the other ~0.75 MB | — |
| Perf baselines (wave-end #1: 11 over) | `perf-baselines` landed at 65f4b5d0a ( perf.sh builds with no slot, then runs under gpu-all; wave-end.sh's `perf` stage). Bisect evidence on `perf-baselines-wip` 6f09fb9ad (`tests/evidence/perf-bisect-20261106/`). sprites-20k 2.22 vs 1.05: one step between 0678686a2 and 70c120c0b, suspect 753e8d9d7 (per-sprite `facing`, depth key, frustum test); point-spot-lights went to the local shadow row; gltf-reference, lights-256, showcase, showcase-capture, spheres-20k(-moving), tilemap-256, materials-physical rise in several steps (10-23→10-25, 10-25→11-01, 11-01→wave end); raycast-record-20k 0.371 vs 0.320; lights-2d-64 has no baseline | To-do, each on gpu-all on an idle machine: measure 8095d4f21, 753e8d9d7, 80143ca5c, 15ca814c4 for sprites, then fix or re-baseline naming the commit; measure d30632f52, b49bb670b, 64b554ae3, 506882e93, 71724b4f8, ebb512c3a, de7809995, b785e540b, af9bed852 for every scene and split the earlier ranges for showcase and gltf-reference; measure raycast's baseline commit; record lights-2d-64 (`OG_PERF_RECORD=1`) | — |
| Crowds at 50k | 24.35 ms a frame (cull 0.30 ms) | prepass 10–13 ms and main pass 10 ms dominate | — |
| Watercolour look | reads as oil: the Kuwahara picks dark low-variance sectors; Bousseau rims made it muddier | a watercolour primitive (Bousseau, MNPR) | Fable: [T-01a1aaad-12fe](https://ampcode.com/threads/T-01a1aaad-12fe-75c0-b190-6d9c79b16005) |
| World polish | round 1 landed at 349f7f3af; round 2 landed (cffc25aa6..abd660e86, ocean-storm and shore-storm re-recorded at c044ba9fd; setts, lake glitter and crest-attached spindrift accepted by art) | done | — |
| og.motion | the jump's pitch is clamped, not fixed at the source; the wave goal idles at 12 s | — | [T-cd152924](https://ampcode.com/threads/T-cd152924-03dd-42d3-a4dc-b5ef6716af67) |
| Distant impostors | no shadow of their own, a view from below draws the horizon frame, cards don't sway (R6's limits) | not yet taken up | — |

**Wrap-up held items, landed (2026-11-06).** Destruction determinism at 042811632..d6d4b9958: the vendored glamx f64 Jacobi solver; the wall test was wrong, not the engine (its 35–65 % band failed at 3–5 radii of a 21-radius sweep with either solver), and now measures each pier's lower half above its anchored course, still ≥ 0.4 m; cross-host hashes agree (physics3d 16912046865738684538, yard tick 427 7bb9d5d5e0a674ea; the b81e819120564fb1 first noted came from lane-6's og built at 02:38, before the branch's commits at 02:39, and no committed tree reproduces it: `og run examples/demolition-yard --headless --size 1280x577 --commands "1.0167:boom;2.6167:strike;3.0167:drop;3.1167:throw;7.1167:dump_world:yh.json;7.1667:quit"` gives 7bb9d5d5e0a674ea at the branch's 7a70eff90 (9335c7894 plus a test file), at main c8c82f851 and there with og.Wind back at version 1, so 2e55d1c26 does not move it: og.Wind is lazy and the yard never registers it). Local shadow cost: the row above. WebGPU water at 53cdd2d4d: Tint refused `dpdyFine` after `below_view`'s return and an unparenthesised `a*k ^ b*k ^ c*k`; og-render `tests/wgsl_rules.rs` now runs WGSL's uniformity and operator-grammar checks in quick; Chrome compiles all 119 modules.

**Implemented, not yet verified**

| Item | Where | Thread |
|---|---|---|
| Wasm build, size, determinism, the browser tiers; destruction's replay hash on wasm and Chrome; the browser checks of video, uncooked load and the platform ABI | the wave's end | [T-01a1a993-ffa2](https://ampcode.com/threads/T-01a1a993-ffa2-70d9-9645-0ad77ddbd60a) |
| Safari's full tiers and web boot: paused by the owner on 2026-11-05 until the features land. WIP `safari-compat-wip` 1ecad14cf (diagnostics applied; a fix keeping probe frames from training auto quality, with its display test). Meridian-loop's title fails alike in Safari and Chrome on a fresh build (2.441 %), so the edge difference is not Safari's; the 17 s fx boot's series is unrun; Firefox sync's first tap waits about 1.5 s on the pre-tap warm pipelines, and prewarm does not help. Handoff: `tests/evidence/compat/safari/handoff-20261106/README.md` on that branch | macrunner | [T-01a1ab98](https://ampcode.com/threads/T-01a1ab98-f3aa-7748-9669-1ffd19538764) |
| Windows: paused for the compatibility phase (the owner, 2026-11-05). Landed: SSR's D3D12 race 8ae87efac, derivatives d54163388, Vulkan water references b71681daa. Handoff: `tests/evidence/compat/windows/round-handoff.md` on `compat-win-web-3-handoff-wip` e72948507. WebGPU water fixed at 53cdd2d4d. Open: E5 17/26 on 144f9b19c (controller, environment, fluids3d, geometry, islands, materials, terrain, volumetric, water miss; desktop controls needed); toon rim (fresh Metal PBR 26/toon 12, Windows 26/13, likely one filtered transition bucket; never relax the assertion); DX12 garment 0.0337, impostor 0.0101, post-adjust 0.0101 on unchanged main; Firefox sync first tap (warm-behind-gate on 1592 ms, off 92 ms; prewarm not better); skinned-hooks DX12 0.0101; G1-7 (DPI on monitor change, fullscreen, 16:10–32:9 safe HUD, raw mouse, physical keys); showcase frame times | studio-win | — |
| Phones (Pixel 10, PowerVR): to-do in the compatibility phase: device loss and recovery (`display.deviceLosses`, visible recovery), G1-4 auto quality rerun (g14.ts), the full-tier rejected promise before water-wake, wire, template playtests and fx `--time 120`; the Android ui tier after UI 1B M2's plain-ground change (no style or scene-HUD backend references). The frame cap is verified (fps30 gives 30.1 fps with GPU headroom); Android gfx tier 212/219; iOS's pipeline mode (Apple stays sync until measured) | §3 | — |
| Wave-end #1's fixes: landed (c3db9a13a..46f27b4c3; references re-recorded at aa4133c66: fx-showcase environment, materials, volumetric and 13 art samples, expressionist-painting and lush-painted-worlds judged right). Was `wave-end-fixes` 47a918167 (generated exports 15135435a, saves e80e792fc keeping a save's own SSR values and og.Wind at version 2, inspector 9e21f92b5, palette weighting after the flat Kuwahara 47a918167, fx-showcase lod 29171983b). To-do on Metal at that head: the fx-showcase full run, meridian jump/slide (0.012–0.013), Ember Vale title and beacon (panel text offset, likely the typography), materials (not bisected); re-record what is judged right (stylise-print, hemisphere-light, environment, volumetric, the listed art looks); judge expressionist-painting (1.4 %) and lush-painted-worlds, moved by the watercolour commits | macrunner | — |
| The wasm budget's meaning (row 44) | the owner | — |

Deferred by the owner on 2026-11-05 to close the engine work: SSSR's follow-ups (the water-edge frame, the GI/AO comparison for indirect specular), FSR's further cost work (TAAU stays the default), and the MPM fluid's look notes and cost. What remains before the engine closes: UI 1B's M2 (the owner, 2026-11-05: finish it), world-polish-2 (un-parked by the owner, [T-01a1aba2](https://ampcode.com/threads/T-01a1aba2-caea-720f-92b6-915146831aae)), the wave-end fixes, destruction's native/wasm determinism, the perf baselines, the local shadow cost, watercolour, GPU particles and the compatibility threads, then one more wave's end.

Deferred by the owner: UI text scale at 200 %, accessibility, the genre
archetypes, the overlay's GPU cost, the WebGL 2 fallback (a study only,
§3), the platform asks' joint review. Ray tracing is a non-goal; Radiance
Cascades were measured and not ported (`docs/design.md` §2.6).

**Wave B, 2026-11-02 (history)** (the owner, 2026-11-02: speed first; macrunner's Metal stays the only pixel judge, authors never touch a GPU; wave A stopped with its work pushed and its findings carried into these briefs):

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

**Paused (the owner, 2026-11-03, holding the cap at 8):** the introduction site, [T-01a1a06d](https://ampcode.com/threads/T-01a1a06d-d946-751e-9eb1-fb73ee1a3538), branch `site` at f9302d39a, pushed and not deployed; it deploys only once the features it shows are done. The owner (2026-11-04, later): order is engine first, then compatibility, then the site. The three audits (site imagery, stylisation, terrain/water/grass) feed engine feature work; the homepage's scenes and media are rebuilt only after the compatibility phase. When it is rebuilt, no rough or placeholder model appears on it (no mannequins, debug maps, low-poly Meshy props at 1024 textures): every model is generated or regenerated at hero quality (dense remesh, 2K or larger PBR textures, normal maps, a clean rig) and passes the art director before it is drawn (the owner, 2026-11-04). Earlier the same day: the site is to be re-planned before any recording or deploy, because some of what it would show still looks too rough; Skyreach Meadow's art-approved close shots (site-scenes c23a4a3be) are kept, but its cinematic footage is not recorded until that plan exists. A fresh thread resumes it once these arrive: site-hero's scene build (then confirm `game.custom.hero.*` from a real publish build), studio-win's per-scene perf and the cold first load on the real host, site-motion in `public/live/motion`, U8's accepted UI showcases in place of the interim og-ui frames, final media for the 45 `check.ts --final` blockers, a stronger characters poster, and the UI grid controls' spacing under the nav.

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
