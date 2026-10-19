# Phase 1 — execution plan

Scope: `docs/design.md` §11 phase 1 ("Core"), 13 weeks from 2026-10-20.
Output: a game author can `npm create @origingame/game`, write TypeScript
against `@origingame/engine`, run it in a browser and headless with `og`, and
the platform can run it in an `engine`-class sandbox. Spike findings
(`docs/spike-results.md`) are the baseline; the spike crate is the seed
of `og-render` and is deleted once its code has moved.

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
- **Not done**: terrains are decoded whole at load, not streamed tile by
  tile; a terrain lies along world x and z (its entity's rotation and
  scale are ignored); edits are asset changes and are not in snapshots;
  scattered copies draw without per-copy alpha fade (they shrink) and the
  scatter is rebuilt on the CPU every frame (4.3 ms for 36,000 copies; a
  GPU placement pass would remove it); lavapipe draws no shore foam on
  `terrain-far`'s 24 km single-quad sea (not investigated).

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
og.platform's answers carry no time, so the idle game cannot know how long
it was closed (`relaunch:<seconds>` stands in for the host); og.audio
reports no output latency and og.Input no time within the tick, so the
rhythm game calibrates the whole chain from the player's taps; an
AudioSource cannot start part-way (a resume restarts the two-bar phrase);
og.ui's `settingsScreen`, `resultsScreen` and `idleHud` carry English
words and number formats of their own, so the settings, end cards and the
idle HUD are built from widgets; `og polish` reads og.ui's phone title
menu (a bottom sheet) as HUD. Two engine defects met on the way: a system
with `og.Rng` in `reads` only gets streams that answer NaN
(`world.resource(og.Rng).value.fork("x").next()`; declared in `writes`
it answers 0.6868…), so the roguelike's systems declare it written; and a
system that reads a resource it did not declare gets a refusal it can
only notice by checking `ok`.

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
owner (39, 40, 44).

After the phase 2 animation and physics round: rows 25, 26, 27 and 43 done
too; 29 (3D SPH) still open.

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
| 12 | `templates/fx-showcase/src/main.ts`'s type errors; templates are not typechecked in the gate | E6 | S | engine, E7 | done, c5295be |
| 13 | Native and Chrome world hashes of the three archetypes compared (match3 differed, not investigated) | E6 | S | engine, E7 | done, 85c2bd0 |
| 14 | The publish wasm under 65 % of the 4 MB brotli budget without removing features (71 % before E7) | E7 | M | engine, E7 | partly: 72.0 % to 70.2 % (fcca471); under 65 % is row 44 |
| 15 | UI sounds start one tick after the input (`og.ui.sound` runs after `og.audio`) | E3 | S | engine, phase 2 | open |
| 16 | Springs for the skin's hover and press (eased curves today) | E3 | S | engine, phase 2 | open |
| 17 | Per-glyph tracking from a skin's ramp (needs per-glyph spacing in og-render's text) | E3 | M | engine, phase 2 | open |
| 18 | A real text glow (an MSDF outline in og-render; four offset copies today) | E3 | M | engine, phase 2 | open |
| 19 | An atlas tool for skins drawn by hand, beyond `make-skins.py` | E3 | M | engine, phase 2 | open |
| 20 | `vulkan-cpu` and browser references for og-ui's scenes, the UI kit, the skin packs and the locale screens (metal only) | E2, E3, E6 | M | engine, phase 2 | open |
| 21 | Pads beyond the first in og-ui | E2 | M | engine, phase 2 | open |
| 22 | Browser runs of og-tests' own gfx scenes (a wasm harness) and GL-tier references (`msaa` among them) | M3, E2 | M | engine, phase 2 | open |
| 23 | Shader hooks on skinned meshes; shadows and the depth prepass displaced by `vertex_offset` | E5 | M | engine, phase 2 | open |
| 24 | Planar water reflection, shadowed water, an underwater view | E5 | L | engine, phase 2 | done, 0ee78f1 (og-render), 5be9281 (og.Water fields), 15ff230 (fx-showcase terrain station) |
| 25 | More than four metaball fluids a frame, fluids that interact | E5 | M | engine, phase 2 | done, bf15ab1, 53fa607, 4a716f7 (sixteen a frame, groups that layer by density and blend by mixing) |
| 26 | Cloth self-collision | E5 | M | engine, phase 2 | done, bf15ab1, 4a716f7 |
| 27 | 2D ragdolls | E5 | M | engine, phase 2 | done, bf15ab1, 53331df, 4a716f7 |
| 28 | A moon disc in the night sky | E5 | S | engine, phase 2 | open |
| 29 | 3D SPH (3D water is the height field) | E5 | L | engine, phase 2 | open (not started in the animation and physics round: left for the next) |
| 30 | Forward+ clustered lights beyond the 8-light uniform loop (CPU tile table on WebGL2) | M3, M6 | L | engine, phase 2 | open |
| 31 | A normal matrix for non-uniform scale, vertex tangents, clearcoat, and a gfx scene of the M6 material features | M6 | M | engine, phase 2 | open |
| 32 | The runtime loaders transcode to the tier's compressed texture format and upload it (RGBA8 today) | M4, M6 | M | engine, phase 2 | open |
| 33 | og-ui's rich text, accessibility tree, glass, two-axis scrolling (`crates/og-ui/README.md` §What phase 2 owes) | M6 | L | engine, phase 2 | open |
| 34 | The M4 "glTF edit to reload in 300 ms" run and the M5 timed 8-frame `sweep`, filed as evidence (`og dev` prints the first) | M4, M5 | S | engine, phase 2 | open |
| 35 | wgpu and naga 31 when published, kept only if every reference frame passes unchanged | Toolchain | S | engine, phase 2 | open |
| 36 | `og polish` measures frame time on the tier it runs on (metal here), not on the executors' lavapipe | E6 | S | platform | open |
| 37 | The eight platform asks (`docs/platform-asks.md`) | E6 | M | platform | done: landed on the platform 2026-10-19, the engine speaks it (96f4ebd) |
| 38 | The platform side of the cutover: merge `origin-engine-cutover`, pin this revision | E1 | S | platform | open |
| 39 | A third native tier beside `metal` and `vulkan-cpu` (the browser tiers stand in) | E4 | M | owner | open |
| 40 | Mobile measurements: the iPhone first frame and frame times (Safari, WebGPU), the mid-range Android view benchmark | M6, E1 | S | owner (devices) | open |
| 41 | og-ui plays no sound when keyboard or gamepad focus moves (ui-kit's sound on input 38 in og polish) | E7 | S | engine, phase 2 | open |
| 42 | The kit's small type: the shop cards' item names (`caption`, 7 CSS px of ink), the idle HUD's generator names and the `meta` units, under og polish's 9 px | E7 | S | engine, phase 2 | open |
| 43 | The default tokens' primary and danger button labels at 3.2:1 and 2.9:1 against their fill (og polish wants 4.5:1; WCAG's large text 3:1) | E7 | S | platform (the design tokens) | done: the platform's tokens of b71ae8fac, og-ui synced with the danger tint in cf932f8 (every role 4.5:1 or more, `crates/og-ui/tests/button_contrast.rs`) |
| 44 | A publish wasm per profile (measured 63.3 % 3D, 59.5 % 2D at opt-level 3): two shipping variants, `og build --publish` choosing by the game's profile, and the size budget read per download instead of as the sum of every wasm the gate builds | E7 | M | owner (the budget's meaning), then engine | open |
| 45 | `og.ui.shake` shown in a template (ui-kit's shop refusing a purchase) | E7 | S | engine, phase 2 | open |
| 46 | runner: a press during the game-over card's hold is silent (og polish: 1 of 3 inputs in `flow`) | E7 | S | engine, phase 2 | open |

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
| M4 Cook | 5–8 | `og cook`: glTF → `.ogb` (meshopt vertex/index codec, cache ordering, LOD chain, tangents), textures → KTX2 Basis (UASTC for normals, ETC1S for colour), audio → QOA, fonts → MSDF atlas, scenes `.ogt` → `.ogb`; incremental by hash; cook stamps checked at load; runtime loaders; hot reload by snapshot + bundle swap | cook of the template and archetypes is deterministic (same bytes twice); a glTF edit → `og cook` → reload shows the change in ≤ 300 ms with world state kept; cooked size of the template ≤ 60 % of its sources. **Filed** (`tests/evidence/M4/`, 2026-10-17): `cook-determinism.json`, `cook-report-template.md` (cooked 35.7 % of sources), `cooked-gltf-reference.png`, `cooked-gltf-reference-compare.json`, `texture-psnr.md`. Not filed: the glTF edit → reload ≤ 300 ms measurement (`og dev` prints it since 5c79c75; §Owed row 34). |
| M5 Probe | 6–9 | `og-probe`: command channel (core actions of design §3.3), module-registered commands, probe protocol 5 members + `capture`, `sweep`, `reload`, `snapshot`, `restore`, `dump`, `commands`; native loopback WebSocket; in-page request ring; `publish` trim | the platform's probe conformance suite (`origin/packages/contracts`) green on both hosts; `og run --commands "0.5:touch_move_to:Start;0.6:touch_start;0.7:touch_end;2.0:screenshot;2.2:dump_world;2.5:quit"` on the template produces the frame and the dump; a `sweep` over a tunable yields 8 frames in ≤ 1 s native. **Filed** (`tests/evidence/M5/`, 2026-10-17): `probe-conformance-test-output.md`, `probe-conformance-transcript.json`, `scripted-run-dump-world.json`. Not filed: the `og run --commands` run on the template (see M6 `tools-template-*`) and the timed 8-frame `sweep` (§Owed row 34). |
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
