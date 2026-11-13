# Origin Engine — design

Normative: the engine's current design. Open work is in `docs/roadmap.md`;
the UI is `docs/ui.md`; the gate, toolchain and reference frames are
`docs/ci.md`. Signatures, fields and defaults are the IDL's
(`crates/og-api/api/`) and the generated wiki's; workflows and thresholds are
the guides' (`crates/og-api/guides/`). This document states the structure,
the contracts and the invariants they rest on.

## 1. Problem and non-goals

**Problem.** OriginGame produces games through an agent that must write, run,
observe and correct a game in minutes. Godot is editor-centric and heavy to
run headless; Three.js is a scene-graph library whose games can be isolated
only inside an iframe. The engine that author needs does not exist, so we
build it.

**Goals.**

1. A full mid-size 3D engine (and 2D as a subset): PBR rendering with shadows,
   IBL and post, skeletal animation and IK, rigid-body physics and character
   control, navigation, spatial audio, particles, terrain, decals, text and a
   UI toolkit. Delivered to the Web first; native store packages and
   mini-game platforms (WeChat first) are goals on the same core and API
   (`docs/roadmap.md` §Direction). Today the native builds serve executors
   and developers.
2. AI-native from the first frame: a command channel, a probe, world dumps,
   deterministic replay, frame capture and parameter sweeps are runtime
   features in every build.
3. Game code in TypeScript against a stable, documented API; no compiler wait
   and no Rust toolchain in the author's loop.
4. Performance that justifies replacing Three.js: the heavy per-frame work
   (culling, draw submission, animation, physics, particles, navigation) runs
   in WebAssembly on WebGPU, not in JavaScript. Batched, instanced and culled
   by default, so the slow version cannot be written.
5. One play surface without iframes.

**Non-goals.** A visual editor as the primary authoring surface (an inspector
and gizmos exist for humans; authoring is code and text). Scripting languages
other than TypeScript. Console platforms. Ray tracing. Networking engines
beyond the platform SDK's rooms.

## 2. Architecture

```
            ┌──────────────── play page (top-level, play origin) ────────────────┐
            │  shell (TS, main thread): input, audio unlock, PWA, platform UI,    │
            │  creator overlay (Stage/Tune/annotations), probe host               │
            │        │ SharedArrayBuffer rings (input, clock)   │ postMessage      │
            │  ┌─────▼──────────────── engine Worker ───────────▼───────────────┐ │
            │  │  origin-engine.wasm (Rust): ECS, tick, renderer (wgpu→WebGPU   │ │
            │  │  on OffscreenCanvas), anim, physics, nav, audio mix,           │ │
            │  │  UI, probe, commands                                           │ │
            │  │        ▲ typed views over wasm memory                          │ │
            │  │  game.js (TS bundle): components, systems, scenes, tunables    │ │
            │  └────────────────────────────────────────────────────────────────┘ │
            └────────────────────────────────────────────────────────────────────┘

            executor (sandbox or the person's machine)
            ┌────────────────────────────────────────────────────────────────────┐
            │  og run --commands "...": bun + the engine's native addon           │
            │  (same Rust core, wgpu→Vulkan/Metal/DX12); same game.js;           │
            │  frames, dumps, sweeps, replays as files                            │
            └────────────────────────────────────────────────────────────────────┘
```

### 2.1 Layers

| Layer | Language | Owner | Notes |
|---|---|---|---|
| Engine core | Rust (`crates/og-*`) | engine | one codebase; targets `wasm32-unknown-unknown` and native (Linux x86_64, macOS arm64, Windows x64) |
| Hosts | Rust + generated glue | engine | `og-host-web` (wasm-bindgen, Worker + OffscreenCanvas), `og-host-node` (napi) |
| TypeScript API | generated `.d.ts` + a thin hand-written ergonomic layer | engine | generated from one IDL (§3); the ergonomic layer is the three.js-flavoured vocabulary |
| Game | TypeScript | the Maker's agent | components, systems, scenes, tunables, assets |
| Tooling | Rust (`og`) | engine | one binary: build, run, cook, probe, check, test, doctor |
| Shell | TypeScript (platform `site/`) | platform | the play page and the creator overlay; consumes the engine |

**Structure rules.** They bind every crate; open work against them is in
`docs/roadmap.md` §Structure.

- **A runtime module is optional.** `og-runtime` is the core (declarations,
  engines, the tick, assets, saves, the probe) plus modules, one per IDL
  module that binds a feature crate (navigation, AI, story, destruction,
  video, the motion model, terrain, the physics profiles). Each module is a
  Cargo feature of `og-runtime` passed through by both hosts, on by default.
  A build without one keeps its binding symbols, its records and its state
  blocks, so a world is the same bytes and the API is the same surface; its
  calls answer a refusal naming the module (`terrain_absent.rs` is the
  pattern). The core never names a module's crate outside that module's
  files and its mount point. This is what a separately loaded artifact, a
  mini-game subpackage and a native package cut along.
- **One host implementation.** A host supplies what differs: its wire types
  (arrays, views, bytes, functions), how the dispatcher is called, the
  surface and the audio sink. Every binding whose body is the host-neutral
  runtime is written once (`og-runtime`'s `host_impl`), never copied per
  host; a third host adds a wire, not a copy.
- **Formats below features.** The cooked formats (`.ogb` layouts, their type
  hashes and readers) live in crates below the renderer, so the cook and the
  bundle reader do not link a feature crate for its types.

**Type checking.** `og check` type-checks game code with tsrs, a pinned
binary run as a child process and never linked (`scripts/install-tools.sh`
installs it); where tsrs has no package (Windows, Intel macOS) it falls back
to tsgo and says so. Diagnostics are values (`--json`: one `check` event).
`og run`, `og test` and `og build` check before they bundle and stop on a
type error (`--no-check` skips it; `og build --publish` refuses it); `og dev`
checks beside each rebuild and never refuses the reload. A checker that
crashes or prints nothing readable is a failure, never a clean check.
Incremental state lives under `.og/check`, locked per configuration and
invalidated by the compiler itself.

### 2.2 Threads

- **Main thread**: the shell only. Collects input into a SharedArrayBuffer
  ring, owns the AudioContext (unlocked once on the first gesture), the PWA,
  platform UI and the creator overlay. Never runs game code.
- **Engine Worker**: the wasm instance, the renderer on an OffscreenCanvas,
  the game bundle. The tick and the frame run here; a slow frame never stalls
  the shell.
- **Lanes**: parallel systems through `wasm-bindgen-rayon` on a pool of
  Workers sharing the wasm memory. Requires cross-origin isolation
  (COOP/COEP) on the play page, which the platform declares per publication.
  Render recording stays on the engine Worker (wgpu's web objects are not
  shareable). Without isolation the engine runs single-threaded with the same
  API. A lane never changes a result: helpers return in item order, and a
  library that splits work itself (rapier's `parallel` build, bitwise
  identical on any thread count) uses the pool only above a grain of work per
  thread.
- **Native host**: the same split (shell = `og`, engine on its own thread,
  lanes via rayon).

### 2.3 Data model

- **World**: an archetype ECS. Entities are `u64` ids with a generation;
  components are plain data declared with `#[derive(Reflect)]` in Rust (any
  reflected type can be a component, resource or event) or
  `defineComponent({...})` in TypeScript, which registers the same reflected
  layout. Storage is SoA in wasm memory.
- **Typed views**: a TypeScript system receives typed-array views over the
  component columns of each matched archetype, one call per archetype per
  system per tick, never per entity. A structural change invalidates them;
  a stale view throws with the tick it belonged to.
- **Resources**: reflected singletons (time, input, camera, tuning).
- **Reflection** is the one schema (fields, types, ranges, labels, units,
  `tunable`, docs), exported by `og schema`, and what the inspector, Tune,
  the probe's `bounds`, scene text and the wiki read.
- **Handles**: assets, meshes, materials, sounds and scenes are generational
  handles; a dead handle is an error value, never a crash.

### 2.4 Time and determinism

- **Fixed tick** (default 60 Hz, per project), an accumulator with a cap.
  Game code never reads wall-clock time: `Time` is a resource with `tick`,
  `dt`, `elapsed`. Rendering interpolates transforms, the camera, skinned
  joints and world-anchored UI between the last two ticks
  (`RenderSettings.interpolate`, on by default); a frame on a tick boundary
  and every probe capture draw the tick, so reference frames are the ticks'.
- **Randomness** is `Rng` resources seeded from the project seed and their
  owner; the sequence is identical on wasm and native (`rng.ts` and `rng.rs`
  share test vectors).
- **Snapshot and restore** cover the whole world (components, resources, rng
  states, pending events). Sweeps, hot reload, `feel_compare` and a game's
  saves (`world.snapshot`, `world.restore`, in every build) are built on it.
- **Restore notification.** `og.WorldRestored { from, tick }` arrives on the
  first tick after `world.restore` and is world state: it is in that tick's
  events, snapshot and `world_hash`. A restored run therefore hashes
  differently from an uninterrupted one on that tick only. Rollback's
  internal rewind emits none. Save/load tests compare equal notification
  histories, or the first resumed snapshots with only that event removed;
  they never ignore every difference on that tick.
- **Floating point**: simulation uses `f32` with the same operations on both
  hosts; the engine ships its own `sin`/`cos`/`atan2`/`exp`. Rapier is built
  with `enhanced-determinism`.
- **Large worlds.** A world that names `og.WorldOrigin` shifts its zero as a
  tick starts, when its focus is farther than `step` from it, by a multiple
  of `step` (`origin.rs`). The shift is world state decided from world state,
  so replays, restores, rollbacks and lockstep peers shift on the same tick;
  the zero is kept in f64 and `og.OriginShifted` carries the delta. What
  moves is declared in the schema (`worldspace world | root | hook | none`,
  in the IDL and the Reflect derive); og-idl refuses a vector field whose
  name reads as a place without a mark. Every `StateBlock` implements
  `shift`. A power-of-two `step` (default 2048) keeps a shift exact in f32.
  The render origin (§2.6) sits on the absolute world's grid, so a shift
  never moves it. A world that never names it keeps its hash and frames.
- **Presentation is outside the simulation.** What is computed only for
  display (the motion model's poses, §2.7; particles; frame history) may
  differ between devices, precisions and runs. It never writes a component
  the simulation reads, is not in a snapshot or `world_hash`, and game code
  does not branch on it; the simulation stays bit-identical on every host.
- **Presentation checkpoints.** A capture is a function of the world, the
  presentation checkpoint and the interpolation fraction, never of how many
  frames were drawn before it: drawing every frame from tick 0, drawing only
  the captured tick, and restoring a mid-run snapshot give the same PNG bytes
  on one host and backend. The parts:
  - *Stepped by the tick.* Presentation state that depends on earlier ticks
    (particle histories, the og.motion Presenter, terrain streaming) is
    stepped as each tick starts, drawn or not (`presentation.rs`); a frame
    reads it and never advances it.
  - *Checkpointed beside the world.* A probe snapshot keeps the presentation
    state with it (particle histories, the renderer's frame history via
    `save_state`, the interpolation pair, skinned palettes, the terrain
    streamer, the motion Presenter), outside `world_hash`, and reports its
    own presentation hash (`SnapshotInfo.presentation`, per host and
    backend). A restore brings it back whole, so later display frames equal
    the uninterrupted run's; a checkpoint the renderer no longer holds
    restarts the history and the first frame draws the tick.
  - *Capture frame history.* Every capture goes through
    `Renderer::render_capture`: temporal history is reset before and after
    it, the phase-driven noise starts at phase 0, and a fixed warm-up
    (one jitter cycle under TAA, `CAPTURE_WARMUP` where GI or reflections
    accumulate) precedes the kept frame. Auto exposure meters the captured
    frame at once.
  - *og.motion.* No model result holds a tick on any host: the Presenter
    steps the ticks in order from each tick's inputs and catches up when a
    result lands. Captures, presentation hashes and fork checks settle first
    (they wait for every result due by their tick), so their frames are the
    same run to run on one host and GPU; live frames may vary.
  - *Captures wait; display frames never do.* A capture draws every lit
    pipeline as its full variant and waits for it to compile; a display
    frame draws with what has compiled. A test comparing display frames byte
    for byte settles the compiles first (`og_render::settle_pipelines`) or
    compares captures.
  - *Capture-only* (`og run --capture-only`) draws only the requested
    captures and equals a run that draws every frame.
  - *Session forks.* `og run --forks N` restores N snapshots taken across the
    run and plays each to the end; every fork must match the run's world
    hash on every tick, its screenshots and its final presentation hash, and
    any difference fails the run. This is GGPO's SyncTest applied to both
    hashes (`packages/engine/tests/presentation.test.ts`).
- **Module state.** Game state in a JS module is in neither snapshot, so a
  restore, rollback, fork or capture-only run would lose it. `og check`
  fails on it (OG9101, `crates/og-cli/src/module_state.rs`): module-scope
  bindings or containers a function writes, and state a closure handed to
  `defineSystem` or `og.game` keeps between ticks. The determinism guide
  lists what it does not see.

### 2.5 Scenes and assets

- **`.ogt`** is the one text format: a line-based `type`/`version` header, a
  separator, then `key value` lines with nested blocks. Scenes, materials,
  tuning, import settings and the project manifest (`project.ogt`, whose
  `budgets { }` `og doctor --project` checks) use it. It diffs line by line;
  the agent's checkpoints are git commits.
- **`.ogb`** is the one cooked blob: type hash, schema version, cook stamp,
  then the reflected struct with its payloads (meshopt vertex streams, KTX2
  Basis Universal textures, QOA audio, MSDF fonts, scenes as component
  tables). Blobs load by copy-free view where alignment allows.
- **Cook** (`og cook`): incremental and hash-stamped. Sources live in
  `assets/`, output in `cooked/`; the runtime reads only `cooked/`. Colour
  textures are ETC1S unless project.ogt's `cook { color_textures uastc }`
  says otherwise (`publish { color_textures etc1s }` overrides it for the web
  export). The base colour texture of an alpha-tested glTF material gets mips
  that keep its coverage at the material's cut-off (og-cook's
  `cutout_mips`); a texture file or a run-time alpha test is not a cut-out to
  the cook.
- Reusable production assets come through the platform's asset MCP; the cook
  consumes what the agent put in `assets/`.

#### Uncooked loads

Content a player makes or a generator writes at run time is read uncooked
from bytes: `og.load.imageData` (PNG, JPEG) and `og.load.gltfData` (a `.glb`,
or a `.gltf` with `data:` URIs). The result draws as a cooked asset does.

- **Budgets before decoding.** `LoadBudget` is checked from the headers
  alone; a load over a limit is refused naming the limit, the value and the
  fix, and nothing is allocated for it.
- **The reader** (og-bundle `runtime::UncookedGltf`) builds the same
  `CookedModel` a cooked blob decodes to. What it skips and what it refuses
  by name is in the loading guide.
- **Off the engine's thread.** Images decode in the browser on the web and on
  a worker thread natively; a glTF's primitives are converted at tick
  boundaries within `frameMs` on the web.
- **Completion is host input.** On the web each load is placed at a tick
  boundary and journaled as `{ request, tick, ok }` (`og.load.completions`);
  `og replay` records and replays the journal. An asset's status is not world
  state. In a lockstep or rollback room a game sends the bytes through the
  room and uses the content only from a tick the peers agree on through
  their inputs, never from `og.load.status`.
- **Release.** `og.load.release` frees an uncooked asset: its GPU and CPU
  copies go and its handle answers failed. Physics shapes built from it are
  world state and stay.

#### Prefabs

A prefab is a reusable entity tree authored once and placed many times. An
instance stores only what differs from its prefab, so a prefab edit reaches
every instance on the next cook and an instance's edits survive it.

**The asset.** An `.ogt` of `type prefab` with exactly one top-level
`entity` block in the scene grammar. Entities inside it are addressed by
their **node path**, names from the root joined by `/` (`""` is the root);
sibling names are unique.

**Placing one.** Anywhere an entity can appear, it may be an instance:

```text
entity "crate1" {
  prefab "prefabs/crate.ogt"        # its tree is the prefab's
  Transform { position 4 0 2 }      # an override of the root
  override "lid" {                  # an override of a descendant
    Health { hp 40 }                # only the fields that differ
  }
  remove "lid/sticker"              # a descendant this instance drops
  entity "loot" { ... }             # an entity added under the root
}
```

`remove "<node>" Health` drops one component. Instances nest to any depth; a
cycle is refused naming the chain. A prefab whose body starts with `base
"<prefab>"` is a variant of that base, and variants chain.

**Cooking.** The cook flattens depth-first in text order: the prefab's tree,
then the variant chain's deltas from the base outwards, then the instance's
own. The scene blob keeps its shape, so a flattened prefab spawns and hashes
exactly as the same entities written by hand, plus a `prefabs` table (source
prefab, node, instance root). A scene's cook stamp includes every prefab it
reaches. An **orphaned** override or `remove` stays in the text so a rename
back restores it: a dev cook warns, `og prefab check` fails, a publish cook
refuses it by name.

**Names.** An instance's descendants are named `<root name>/<node path>`; a
collision fails the cook naming both.

**At run time.** Each spawned entity carries `og.PrefabLink`, ordinary world
state carried by snapshots and the hash. A game without prefabs has none.

**Write-back (dev builds).** `save_scene` keeps instances as instances,
writing a tuned field as an override only when it differs from the prefab.
`apply_prefab` writes an instance's overrides into the prefab's text (or,
with `:base`, a variant's base) so the scene cooks to the same world;
`revert_prefab` removes them. `og prefab check | show | extract` are the
tools.

**Determinism.** Flattening is a pure function of the texts.

#### Sub-scenes and world cells

Streaming groups (§2.6, IDL `stream`) move what a frame draws and plays;
nothing a tick reads depends on them. World cells move entities, which a
tick does read, so what is loaded at a tick is a function of world state
alone, never of I/O timing.

- **Sub-scenes.** `scene "<id>"` in an entity block places another scene
  there as its own blob; a cycle is refused naming the chain.
- **Cells.** An entity with a sub-scene and `og.StreamCell` is a cell. As
  each tick starts (before any system, so replays, re-simulations and
  restored worlds run it), the engine decides from `og.StreamAnchor`s (or
  the camera, refused under lockstep) whether each cell is loaded, spawns or
  despawns its members (`og.CellMember`), writes `loaded` and sends
  `og.CellEvent` in the same tick. Cells are visited in entity order and
  spawn in blob order, so the same world allocates the same ids on both
  hosts, in a replay and after a restore.
- **Saves.** Loaded cells and their entities are ordinary world state. A
  `persist` cell keeps its members' rows in the `og.cells` state block across
  unloads; `budget` caps spawns per tick, with the load in progress in the
  same block. The cell records are lazy: a world without cells keeps its
  type table, saves and hash.
- **A cell that is not ready.** A sub-scene in a cell group streams; a tick
  that would load a cell whose blob has not landed is held by a gate (never
  skipped or reordered), which natively never holds. `dump_stream`'s `hold`
  and the log report holds.
- **Baked colliders** (`model { collider true }`) are used only while the
  mesh is bit-for-bit the one baked, so nothing a tick computes changes.
- **Texture mips** (`texture { stream true }`): the boot bundle carries the
  mip tail and the whole chain streams by on-screen footprint within a mip
  budget. Presentation only; `mip_stream:resident` makes captures whole.

### 2.6 Renderer

wgpu with WGSL shaders cooked per backend (naga). On the web the renderer is
WebGPU only today (§8). A capability-limited GL tier, for mini-game platforms
without WebGPU and older devices, is planned (`docs/roadmap.md` §Direction):
features answer it through capability queries, and the WebGPU tier is never
lowered to meet it. Implementation details of each pass are in
`crates/og-render/README.md`; fields and options are in the `render` IDL.

**Tiers.** A tier is a declared set of device capabilities, never a quality
setting. The full tier (WebGPU, Vulkan, Metal, DX12) has compute, multiple
render targets, asynchronous pipeline creation and timestamps. A limited
tier lacks some: the mini-game WebGPU tier has no MRT, asynchronous
pipelines or timestamps; the GL tier has no compute either. Three rules
follow:

- Within a tier nothing is reduced for a budget (§7).
- A feature states the capabilities it needs; on a tier without them it is
  absent and says so through `RenderCapabilities`, as `world3d` does in the
  2D profile. It is never a degraded copy.
- The renderer asks capabilities, not backends: a new tier is a capability
  set plus the passes that have a path without the missing ones, never a
  second renderer.

Quality presets (`low`, `medium`, `high`, `autoQuality`) are a player's
choice inside a tier and unrelated to this. The CPU culling and particle
paths remain options on every device. The engine is GPU only on every host:
a player's device without a GPU is not supported and no feature adds a CPU
fallback for one. Developer tooling runs without a GPU: the logic tests and
the probe's CPU description of a frame.

**What it draws.** Each item is specified in og-render's README and the IDL.

- Forward+ (clustered), with a depth-normal prepass only when an effect reads
  it; glTF metallic-roughness PBR and three.js's physical lobes.
- Order-independent transparency for the main camera (weighted blended with
  each pixel's nearest layer exact; per-pixel lists as the high tier).
- Cascaded shadows for one directional light and cube or dual-paraboloid
  shadows for up to 8 local lights. The cascades are nested round the
  camera and snapped to a grid, so a camera that turns keeps every
  cascade's view and one that walks moves it in steps; a shadow layer
  holds what stood still, and its static casters are kept apart and put
  back under the moving ones only where that costs less than drawing
  them again (`shadow_fit.rs`, `shadow_cache.rs`, `shadow_keep.rs`).
- IBL from cooked or run-time environment maps, the sky, height and
  volumetric fog, baked lightmaps and light volumes, screen-space and probe
  GI, reflection probes and SSR (§10.6, §10.7).
- Post: bloom, tonemapping, grading, AO, antialiasing (TAA by default in 3D,
  4× MSAA by default in 2D) and temporal upscaling (§11).
- Stylisation as data a look composes (`og.Stylise`, `og.MaterialStyle`,
  `type look`); every primitive is off at zero and then free.
- GPU skinning, instancing by default (the API has no per-object draw call),
  GPU particles, terrain, decals, MSDF text with run-time and colour glyphs,
  2D lighting with shadows and normal maps.
- **Graphics quality is data.** RenderSettings' quality fields are what a
  frame may cost; a preset (`og.render.applyQuality`) is a row of them, with
  no hidden state behind the fields. The device caps some and says so in
  RenderCapabilities.
- **three.js parity for ports**: opt-in fields that reproduce what three.js
  and its common passes draw, each off by default (the three-to-og guide).
- Capture: a readback path from the render graph used by the probe on both
  hosts; on the web it reads the OffscreenCanvas target.

**Meshlets** (`meshlet.rs`). The cook cuts every model level into clusters of
at most 64 triangles and keeps their draw order in its own stream
(`cluster_indices`). The simulation mesh, its raycast BVH and its index order
are never reordered, because a rendering optimisation must not change ray
hits or contacts. Every cluster carries its own and its parents' error so
exactly the level `lod::select` picks is drawn, and the GPU and CPU paths
draw the same picture. Visible clusters are compacted in a stable order, so
neither draw order nor equal-depth software triangle ids depend on workgroup
scheduling. The camera's small opaque clusters rasterise in compute into a
visibility buffer and are shaded by the same lit functions as the hardware
path. `cook { clusters false }` cooks none. The CPU statement of every test
is `meshlet.rs`, whose unit tests check it.

**Runtime geometry, lines and debug draw** are §10.1.

**Debug views never substitute the beauty image.** An unsupported
surface/view pair draws an eight-pixel magenta/black checker, and
`dump_render.debugCoverage` lists the supported paths. Debug draw is not
world state, never feeds the tick, and the `debug` module is left out of
publish builds.

**Render targets, views, layers and custom shaders** are §10.8.

**Render origin.** The frame is drawn relative to the point of an 8192 m grid
nearest the camera (`og-render/src/frame/origin.rs`), so the GPU's f32 arithmetic
works on small numbers and `p - origin` is exact. Everything the renderer
keeps between frames moves with the origin, so a move is invisible;
patterns laid in the world add it back in f64. The origin never reaches the
tick; keeping the simulation precise is `og.WorldOrigin` (§2.4).

**Shared frames.** A native engine process can show its frames in another
process's window with no CPU copy (Origin Desktop's Preview). With
`ORIGIN_FRAMES_SOCKET` set, the native host links its renderer to that
socket (og-render's `shared_frames`): shareable textures per generation
(IOSurfaces, Vulkan opaque fds, D3D12 NT handles) announced with `Hello`,
each frame copied on the GPU into a free slot and announced `Ready`,
`Release` returning slots and `Resize` answered by `Rehello`. The wire is
the desktop's `shared-frame` contract (origin-agent
`desktop/crates/shared-frame`); neither side links the other. The engine
never waits for the window: a frame with no free slot is not shown there,
and a window that is not listening leaves the process drawing offscreen.

The socket carries two opt-in extensions; messages are a `u32`
little-endian length followed by serde JSON, and nothing is sent without a
subscription.

- **Accessibility.** After `Hello` the window sends
  `{"AccessibilitySubscribe":{"version":1}}`; version 1 pins AccessKit
  0.25.2's `TreeUpdate` and `ActionRequest`. The engine sends
  `{"AccessibilityUpdate":{"version":1,"generation":g,"seq":s,"update":<TreeUpdate>}}`,
  a full tree first and then changes; the window sends
  `{"AccessibilityAction":{"version":1,"request":<ActionRequest>}}`. Node
  ids follow UI paths and are stable within a connection; bounds are the
  shared frame's physical pixels; every update is applied in socket order.
  Actions are applied in the next UI tick after checking focus scope,
  disabled state and target. Origin Desktop owns the platform adapter
  (platform ask 25).
- **Cursor.** After `{"CursorSubscribe":{"version":1}}` the engine sends
  `{"Cursor":{"version":1,"keyword":"<css keyword>","image":null}}`, or an
  `image` of `{width,height,rgba (base64 RGBA8 sRGB, straight alpha),hotspot}`,
  once for the current shape and again whenever it changes (the UI's
  resolved CSS `cursor`, docs/ui.md §2).

A reconnection resets both and requires a new subscription.

### 2.7 Animation, physics, navigation, audio, UI

- **Animation**: glTF skins and clips, blend trees and state machines as
  data, root motion, additive layers, two-bone and FABRIK IK, blendshapes.
  The cook reports skin binding diagnostics (`skinDiagnostics`) as
  information and changes nothing (the animation guide).
- **Destruction** (`og.destruction`): props are prefractured in the cook,
  never at run time. Bond damage, the support graph and Blast's stress
  solver (ported to f32 with fixed orders) run in the tick before physics;
  bond health and chunk entities are a state block. Fine debris is visual
  only and never becomes bodies.
- **Motion generation** (`og.motion`, og-anim's `motion`): game code queues goals
  (a pose, a place and heading, a turn, a walk or run, a jump) and the
  engine's motion model, a student distilled from NVIDIA MotionBricks, fills
  in the motion. It runs in Rust with no third-party inference runtime, on
  wgpu compute by default or on the CPU lanes (`og.motion.setBackend`). Its
  poses are presentation (§2.4): the simulation moves the character by its
  controller, the pose follows, and no component, goal, tick, snapshot or
  hash depends on the model. `motion.presentation` depends on GPU timing and
  game logic must not branch on it. Its weights are an optional pack, never
  in the boot bundle.
  - **Jumps: the simulation owns the elevation.** `motion.jump` takes off in
    the tick after `prepare`: through an og.CharacterController's own jump
    (ceilings and ledges are the controller's) or, without one, a fixed
    ballistic arc. Takeoff, height, flight time and landing are never read
    from the model, whose vertical arc is not reliable; the jump's state is
    in snapshots and the hash. The generated clip is drawn in place on the
    simulation's flight, retimed onto it, and adds no vertical travel.
  - **Layering.** A drawn frame is the generated pose, then the authored
    performance (`motion.react`, `motion.lookAt`), then the feet planted on
    the generated contacts, which outrank both; the root is the Transform
    throughout.
- **Physics**: rapier3d (rapier2d for 2D projects) stepped inside the tick,
  with a kinematic character controller.
- **Navigation**: navmesh baked at cook time or at run time; path queries and
  crowd steering as engine systems.
- **Audio**: the mixer runs in wasm; on the web it feeds an AudioWorklet
  through a SharedArrayBuffer ring, natively cpal.
- **UI**: designed in `docs/ui.md`. Immediate-mode, rendered by the engine,
  with stable string element ids the command channel targets. A game's UI is
  the game's, never the platform's: no platform token reaches it. It is
  dressed in a **style** (`type style`), one complete asset named by a
  look's `ui` line; without one the UI draws the engine's default style,
  generated from `packages/design`. Contrast checks run over every built-in
  style.
- **Timeline and paths** (§10.2): every value a timeline sets is a function
  of its time alone, a whole number of thousandths of a tick, so seeking to
  t and playing to t give the same world hash and the video producer (§2.9)
  starts at any frame. A camera track owns `og.Camera` while it plays and
  hands back the camera it found.
- **2D skeletal animation** (og-anim `rig2d.rs`): `type rig2d` rigs, written
  by hand, imported from DragonBones or rigged by the cook from layered art,
  played by `og.Skeleton2D`. The pose is a function of the component, its
  Transform and its IK targets through arithmetic-only trigonometry, the
  same bits on both hosts. Spine's format is not imported (its licence).

### 2.8 Platform SDK as host ABI

The engine imports a small set of host functions, each a capability the shell
grants (`og_net_fetch`, `og_store_*`, `og_leaderboard_*`, `og_room_*`,
`og_telemetry`, `og_point`, `og_ai_mark`); in TypeScript they are
`og.platform.*`. The shell owns every credential; game code, in the engine
Worker, has no DOM and no cookie. The Worker's own globals (`fetch`,
sockets, storage) remain in scope: the engine does not remove them, `og
check` reports each use (§6), and what confines a game on the web is the
page that hosts it, which is the platform's.

**Providers.** The host side of this ABI is a public contract,
`@origingame/engine/provider` (`packages/engine/src/provider.ts`), so the
OriginGame Player is one provider among others and a game's code is the same
under each.

- A **platform provider** answers `og.platform`'s calls: it lists the methods
  it answers (`capabilities`), takes each tick's calls as a method name and
  JSON arguments, answers each once, and may send notices and pause or
  resume the game. A **room provider** carries og-net's room protocol
  (`crates/og-net/src/protocol.rs`): ops out, events in, in order.
- The contract has a version (`PROVIDER_VERSION`). The method vocabulary has
  **core** methods any store or portal can answer (`store.*`,
  `leaderboard.*`, `telemetry`, `purchase.*`, `ad.*`, `achievement.*`,
  `player.get`, `friends.list`, `invite.*`, `share`) and **host extensions**
  one host defines (`point` and `ai.mark` are OriginGame's). A provider
  answers any subset; the rest is `unavailable`, and `og.platform.supports`
  tells the game.
- Providers in the tree: the Player (protocol 2, `shell/platform.ts`), the
  local document (the native host's file, a dev page's storage, the
  deterministic fake), and for rooms the Player, the native relay client and
  the web relay client (`relayRooms`). A portal, a store or a mini-game
  platform is one more provider, outside the engine.
- For every provider: the provider owns every credential and URL, game code
  names neither, ids are opaque and game-scoped, and a published page never
  fakes money. The page that hosts the game chooses the provider
  (`play({ provider })` or `window.__OG_PROVIDER`), never game code
  (docs/web-export.md §Providers).

**Rooms.** The engine never opens a socket: it queues ops and its host carries
them (`og_room_take`, `og_room_deliver`, `og_room_set_host`). Ops: `create`,
`join`, `quick`, `leave`, `send`; events: `joined`, `peer_joined`,
`peer_left`, `message`, `left`, `error`. The rules:

- **Quick play** (`quick`) is matchmaking by game, mode and player count;
  natively `og relay` keeps the queue, on the web the platform does.
- **Peers** are numbered in join order from 0, never reused.
- **Ordering.** Messages from one sender arrive in the order sent, and its
  departure after its last message; lockstep's agreement on a leaver's last
  input rests on this.
- **A lost connection** is mended by the relay clients: they reconnect with
  backoff for 30 s and rejoin under their name, naming their old number
  (`was`); the engine hears `joined` again, and lockstep gives that peer's
  seat back (§2.10). Sends made meanwhile are lost; `left` comes only when
  the window passes or the room refuses. A silent connection is pinged after
  3 s and dropped after 10 s.
- **Events in the tick.** Events arrive at the start of a tick, like host
  input (og.RoomJoined, og.RoomPeerJoined, og.RoomPeerLeft, og.RoomMessage).
- **The engine's channel.** A channel starting with `og.` is the engine's;
  its messages never reach the game.
- **Limits.** A game message is at most 64 KiB, an engine message 16 MiB.
- **Hosts.** Natively `og relay` serves rooms by URL path and `og run --room
  <url>` joins one (`ws://` or `wss://`, TLS through rustls with bundled
  roots); the relay can simulate a bad network. On the web the shell's
  `RoomHost` maps ops onto the Player's `room.*` (docs/platform-asks.md
  #16); a Play link `?room=<id>` makes the engine join that room itself.

**Purchases, ads, achievements, identity and share**
(docs/platform-asks.md #19) are requests on the same channel answered with
game-scoped opaque ids; accounts and money stay the platform's.

- **Notices.** Unasked host input is applied at a tick's start as
  `og.PlatformNotice` and logged with the answers.
- **Grant once.** `og.platform.grant` records a delivered purchase in
  `og.PlatformGrants`, world state, so a consumable is delivered once across
  restores, replays and relaunches. It, a save or a log is never proof of
  payment: entitlements are always the host's answer.
- **Ads.** The host pauses the game while an ad shows and answers after the
  resume.
- **Sessions.** In a lockstep or rollback session a system that writes
  shared state hears no platform answer or notice: they are this peer's
  alone.
- **Replay.** `og replay` keeps every answer and notice with its tick.
- **The fake.** Natively and in playtests a deterministic fake answers
  (`og run --platform-fake`, `PlaytestDesc.platform`); a published page
  never fakes them.

### 2.9 The video producer

A video is a command script played headless and recorded, built on the fixed
tick, exact seeking by snapshot, headless runs and the offline mixer.

- **Frames.** `og render` runs the game on the native host at its own tick
  and draws the world once every `rate / F` ticks (F must divide the tick
  rate), so every frame is a tick, never between ticks. Each frame is the
  frame the loop drew, read back once.
- **Audio.** `--audio` renders the capture mixer from tick 0, the same bytes
  every run, muxed into the video.
- **Encoding.** ffmpeg is found through `OG_FFMPEG` or `PATH` and reported by
  `og doctor`; without it the render writes its frames and says so.
- **Determinism is the contract.** `frames.json` lists every frame's tick,
  time and the SHA-256 of its RGBA bytes and the audio's; two runs of one
  script give the same list. A changed hash is a changed frame.
- **Jobs.** `--jobs N` splits the range; each part restores a leader's
  render snapshot and warms up before its first frame, so the frames equal a
  one-job render's.
- **Review.** `--contact-sheet` writes a labelled grid; `--timeline <name>`
  plays a timeline over its duration (§2.7). Agents iterate small and review
  the sheet.
- **Footage.** A `type video` asset cooks to frames in a texture array or, with
  `stream true`, an AV1 stream decoded by rav1d natively and WebCodecs (or
  rav1d in its own Worker) on the web; a stream is never in the startup
  bundle. `og.VideoPlayer.time` is world state (f64) advanced by the tick,
  and its sound is an `og.AudioSource` started at that time, so the tick is
  the one clock; the time scale and hit stop leave footage alone. Decoding
  is presentation: an executor waits for each player's exact frame before
  drawing, so captures are deterministic; a browser shows the newest frame
  decoded.
- `og render` and its capture members are development members, absent from
  `publish`.

### 2.10 Lockstep

Deterministic play over a room (IDL module `net`, crate `og-net`). Every peer
runs the whole world; only the players' inputs and the world's hash cross the
room. It rests on §2.4's determinism and snapshot.

- **Settings.** `og.Lockstep` belongs to this peer's session: never hashed or
  compared.
- **Inputs.** `og.PlayerInput` (or the game's own record through
  `og.net.shape`, fingerprinted in the start message) is written at the start
  of each tick from each seat's input; its packed bytes cross the room. A
  game submits its input with `og.net.submit` from the only system that
  reads `og.Input`; it acts `inputDelay` ticks later, and every other system
  reads the seat inputs. Commands (`og.net.command`) act on one tick and are
  never predicted.
- **Start.** Until the room holds `players` peers each world runs freely.
  Then the authority (the lowest peer number) starts lockstep; every other
  peer restores the authority's snapshot and seats go in number order.
- **The tick gate.** Tick t runs only when every seated player's input for t
  is in (og-probe's `set_gate`); a frame that cannot run a tick runs none.
- **Rollback** (`og.Lockstep.mode rollback`): a tick may run on predicted
  inputs up to `maxPrediction` ticks past the last confirmed one; a wrong
  prediction rewinds to that tick's snapshot (no `og.WorldRestored`) and
  runs the ticks again. Only final ticks (run on real inputs) are hashed and
  compared, so the confirmed world equals lockstep's for the same inputs.
  Presentation is deduplicated outside the tick, and `og.net.resimulating`
  tells a game's own effects to hold.
- **Stall, drop and rejoin.** A silent peer is reported (`og.PeerStalled`,
  session-only), and past `stallTimeout` or gone its seat is dropped at an
  agreed tick, so `og.SeatDropped` arrives at one tick on every peer. Wall
  time decides only when a peer is reported and dropped. A returning peer
  (with `rejoin`) gets its seat back from a snapshot, and `og.SeatRejoined`
  arrives at one tick everywhere. Both seat events are hashed and lazily
  registered.
- **Late join.** A joining peer receives a snapshot and takes a free seat (or
  watches) from the tick named in the `admit` message.
- **Hashes.** Every tick each peer hashes its world without its own session
  (input resources, `og.Lockstep`, input, room, platform, asset and stream
  events, UI focus). Hashes are exchanged in batches so the first divergent
  tick is exact; on a mismatch `og.Desync` is sent once and the peers
  exchange snapshots of that tick, which the probe's `dump_desync` diffs by
  entity, component and field.
- **Scripts in a room.** A playtest with `PlaytestDesc.room` runs several
  native processes through a relay; once lockstep runs, a script's steps sit
  on the room's shared ticks.

### 2.11 The model review

A model built as code from a reference image is judged pass by pass; the
engine does the mechanical half. The command channel's
`view:<entity>:front|back|side|top|orbit:<deg>` and `turntable:<entity>:<n>`
frame the entity from fixed cameras; the view is render state, never written
to og.Camera, so no snapshot, replay or hash sees it, and it is not built
into publish. `og compare <reference> <render>` prints silhouette IoU and
CIEDE2000 differences and writes one comparison sheet, the same numbers and
bytes on every run. Thresholds are the art-review guide's.

## 3. API surface

### 3.1 One IDL, two hosts

The public API is declared once in `crates/og-api/api/<module>.ogt`: modules,
functions, desc structs, enums and events, with docs and ranges. From it the
build generates the Rust trait impls of both hosts (checked at compile time),
the TypeScript `.d.ts` and binding tables, the wiki pages
(`packages/knowledge/wiki/`) and the JSON schema the platform's knowledge
package consumes.

### 3.2 Shape of the TypeScript API

```ts
import { og, Vector3, Quaternion, defineComponent, defineSystem, f32, u32 } from "@origingame/engine";

export const Health = defineComponent("Health", { hp: f32(100, { min: 0, max: 1000, tunable: true }), max: f32(100) });
export const Velocity = defineComponent("Velocity", { v: Vector3() });

export const Move = defineSystem("Move", { reads: [Velocity], writes: [og.Transform] }, (q, time) => {
  for (const a of q.archetypes()) {            // one call per archetype
    const p = a.column(og.Transform).position; // Float32Array, stride 3
    const v = a.column(Velocity).v;
    for (let i = 0; i < a.count; i++) {
      p[i*3] += v[i*3] * time.dt; p[i*3+1] += v[i*3+1] * time.dt; p[i*3+2] += v[i*3+2] * time.dt;
    }
  }
});

export default og.game({
  tick: 60,
  scenes: { main: "scenes/main.ogt" },
  systems: [Move],
  commands: { spawn_wave: (world, args) => spawnWave(world, Number(args[0])) },  // registered into the command channel
});
```

Rules the API enforces: desc objects whose omitted fields are the defaults;
every failure is a returned `Result`-like value with a reason; no global
mutable state in game code (the world and resources are passed in); vectors,
quaternions, colours, transforms, material parameters and `og.load.gltf()`
follow three.js naming, while the ECS, the tick and determinism are the
engine's.

**The math layer** is three.js's methods as pure functions over the plain
structs, declared in the IDL's `math` module (`host false`) and implemented
in TypeScript (`packages/engine/src/math.ts`). No classes and no mutation of
arguments, so a result can be written straight into a column, and only
`+ - * /` and `sqrt`, so a system using it stays deterministic.

### 3.3 Command channel

`og run --commands "t:action[:args];..."` and the probe's `commands(script)`
schedule actions on sim time. Core actions ship with the engine; a module or
a game registers more. Unknown actions warn and continue. The core actions
(`screenshot`, pointer and keyboard input, `teleport`, the `dump_*` family,
`set`, the clock controls, `describe`, `expect`, `save_scene`, `quit`) are
listed with their arguments by `og` command help and the testing guide.

### 3.4 Probe

The engine implements `RuntimeProbe` protocol 5 natively (`hello`, `tick`,
`state`, `advance`, `pause`, `resume`, `input`, `accepted`, `pick`, `entity`,
`bounds`, `patch`, `run`) plus the wire-only members `capture`, `sweep`,
`reload`, `snapshot`, `restore`, `dump`, `commands`. Transport: in-page calls
from the shell through a request ring to the Worker, and a loopback WebSocket
from `og` natively. `publish` builds keep the player-facing members (`hello`,
`tick`, `state`, `pick`, `entity`, `input`, `capture`) and compile out the
rest with what only they read; their command channel keeps the actions og.ai
and timeline events reach and refuses the observing ones with a warning
(crates/og-probe/README.md §Builds).

Above capture sits the layer that lets an agent check a frame without reading
pixels:

- **The semantic frame.** `describe` answers the tick's frame as JSON,
  deterministic for the same world: visible entities nearest first with
  screen boxes, pick points, depth and occlusion, the UI elements, the
  camera, and each secondary view with what it shows. Entities come from an
  entity-id pass drawn on the CPU, so a host without a GPU describes the same
  frame; `pick` answers from the same pass when the game has no picker.
  Visibility is per camera, never merged.
- **Expectations.** `expect:<op>` runs as an action or a probe program step
  at its tick (entity boxes, events, UI text, regions, reference frames,
  `state()` facts, entity fields, timings). Each result keeps its tick, what
  was expected and what was observed; `og run --json` carries them and exits
  1 on any failure. A frame expectation without a renderer is unchecked,
  never a failure.
- **Trace.** `og run --trace` writes the Chrome trace format on both hosts
  (engine, render, gpu where timestamps exist, load, cook and lane threads);
  the live probe's `state().timings` carries the last tick's systems and
  frame's passes.
- **Scene write-back.** `save_scene` rewrites every scene the world was
  assembled from, line by line: a field line changes only when its value,
  rounded as the cook rounds it, differs; comments, order and spelling stay;
  resources are scene text too. With `spawned`, run-time entities are
  written under a marked section of the boot scene, regenerated whole. Cooking
  the result loads the same world, and saving again changes nothing.
- **Diagnostics as data.** `og cook --report` writes per-asset figures read
  back through the runtime's loaders; `og doctor --project` checks them
  against project.ogt's budgets and prints findings with severity, asset,
  budget, limit, value and fix. Warnings never modify an asset.

### 3.5 Hot reload

`reload` snapshots the world, swaps the game bundle, re-registers components
by name and restores; a changed layout is migrated field by field through the
schema, and a removed component is dropped with a warning. For engine
developers in a checkout, `og dev --engine` takes the same path one level
down: the dev addon is rebuilt and a new process restores the handed-over
world. A failed build keeps the old game.

### 3.6 The inspector and gizmos

A development build carries an inspector for humans (og-runtime's `inspector`
module, compiled out of `publish`), shown with F1 (Ctrl+F1 when the game binds
F1) over the game's frame in the engine's default style. It reads only the
reflected schema and writes only through the probe:

- **Panel.** Entities and resources with an editor per field. Every edit is
  the probe's `set`: checked against the schema, applied between ticks,
  undoable, marked unsaved. Save is `save_scene`.
- **Prefab overrides.** A prefab instance shows its prefab, node and
  overrides; each can be applied into the prefab (`apply_prefab`) or
  reverted (`revert_prefab`).
- **Where Save goes.** Natively the engine writes the files. A web page
  writes none: the save answers the texts, and the shell sends them to a
  Stage that lists `scene_saved` or downloads them (`shell/wire.ts`
  `routeSavedScenes`; `docs/platform-asks.md` #22). Apply and Revert work
  only where the engine reads the project's files (#23).
- **Gizmos.** Translate, rotate and scale on the selected og.Transform,
  writing through the same `set`, one undo step a drag.
- **Observable.** `dump_inspector` answers the panel's elements and each
  handle's grab point, so a script presses what a person would.

## 4. The play surface without iframes

The platform serves a top-level Player on the play origin
(`play.<domain>/g/:slug`), and it is the only mode:

- Desktop and the creator Stage use the same top-level page; the creator
  overlay renders in the play page itself.
- The play origin carries no Portal cookie; the shell holds a game-scoped
  player token. Game JavaScript runs in the page, which has nothing beyond
  the game's own scope to steal; the host ABI is the only route to platform
  APIs.
- COOP/COEP on the play page enables lanes, declared per publication.
- The shell is the brand surface: boot splash, real progress,
  ready-to-interact gate, safe areas, one audio unlock, PWA install.

## 5. Executors and the sandbox

| Class | Contents | Image estimate |
|---|---|---|
| `engine` | bun, the engine (wasm + native addon), git | ≈ 400 MB |
| `probe` | the same plus ffmpeg (minimal static build) | ≈ 600 MB |
| `asset` | plus Blender headless (trimmed) and the cook tools | ≈ 1.8 GB |
| `browser-check` (evaluation only) | plus Chrome headless-shell for the final "runs in a real browser" check | ≈ 800 MB |

No Godot, no Chromium in the inner loop, no Rust toolchain anywhere in the
sandbox. An image installs the released version the platform pins, the
signed archive for its target (`docs/release.md` §1), never a layout built
from a git revision or a checkout. Sweeps, replays, determinism and the
probe's dumps run through `og` on the native addon without a GPU. Frames need
a GPU: without an adapter a playtest's frame checks are reported unchecked,
never passed. Performance truth comes from the person's device through Player
telemetry and from `og run --device` when the person is present.

**The resident session.** Headless `og run`, `og test` and `og ui`'s check,
preview and sheet start a local host when none is listening (`og session`
runs it in the foreground; `OG_SESSION=0` keeps a fresh process). A session
owns one host per worktree and retires when idle, under memory pressure or
after a fixed number of admissions. Every run admitted freezes its inputs and
starts from fresh declarations, a fresh world, platform fake and audio sink;
only `store: "keep"` shares a playtest's store. Room peers use their own
processes because they advance together. Lost replies are failures, never
retries; an accepted run is never retried. Native bundling is cached under
`.og/bundle` by a key over every input, and each caller gets its own copy.

**Playtests.** A playtest file is a `bun:test` file whose tests await
`runPlaytest`. A failed one reports `failureCaptures` (world dump, save and
the last drawn frame); an unavailable frame is reported unavailable, never
fabricated. These saves are diagnostics, not presentation checkpoints.
`og test --affected <rev>` selects suites by changed source dependencies;
anything unclassified or unresolved selects, unselected suites count as
`skipped`, and an invalid revision fails.

## 6. Failure modes

| Violation | Behaviour |
|---|---|
| stale typed view used after a structural change | throws with the owning tick and the system name |
| dead handle | error value `{ok:false, reason}`; never a trap |
| system writes a component it did not declare | refused at registration with the field name |
| wall-clock read, `Math.random`, a timer, `fetch` or the browser's storage in game code | reachable, reported: the engine hides none of JavaScript. `og check` reports each use as a warning with its replacement (og.Time, og.Rng, `math`, og.platform, og.load) and never refuses; `// og-ok` marks a line that only draws (`crates/og-cli/src/lockstep_lint.rs`, the determinism guide §1). What a game reaches that way is outside replays, saves and the platform |
| wasm trap (engine bug) | the shell shows the crash card with the last 200 log lines and the world dump; telemetry uploads the report |
| WebGPU unavailable | not supported: the player shell says so instead of starting the engine (§8) |
| cook stamp mismatch at load | refuse with the asset path and the two stamps |
| probe refusal | `false`/`null` exactly as protocol 5; `error` only for malformed calls |

## 7. Budgets

`scripts/budgets.txt` holds the gated ones (docs/ci.md); the rest are
measured by hand. Quality comes first: the engine targets current devices
only (§8), and within a tier (§2.6 Tiers) no budget is a reason to cut a
feature or hold a model or an effect below the quality it needs. A figure a
feature moves is pursued, or the budget is raised in its own commit.

| Budget | Target |
|---|---|
| engine wasm, brotli | ≤ 16 MB per download (the largest variant a player loads): a ceiling against accidental growth, not a figure to cut features for. Model weights and other assets are not in it |
| first frame, a current iPhone, 4G, small game | ≤ 2.0 s click to first frame |
| first frame, iPhone 13, Wi-Fi, mid-size 3D scene | ≤ 1.5 s |
| TS system over 10,000 entities (one archetype, 2 components) | ≤ 1.0 ms per tick on a current iPhone |
| reference scene (`crates/og-tests/reference-scene`) | reported by `packages/engine/bench/lanes.ts`, native and in Chrome |
| probe `capture` 1280×720 | ≤ 30 ms native, ≤ 60 ms web |
| hot reload | ≤ 300 ms from bundle write to first new frame (the type check runs beside it and is outside it) |
| headless executor image (`engine`) | ≤ 600 MB |

The wasm budget is per download: a page downloads one engine module (its
physics profile's publish variant, or the lanes variant on an isolated
page), so each module is held to it, never the sum (docs/web-export.md
§Variants). The 2D profile leaves the 3D world out (`RenderCapabilities.world3d`
false, its calls refused by name), so `og build --publish` refuses a 2D
project whose scenes use it, by scene file and line.

## 8. Platform matrix

| | WebGPU (Chrome, Edge, Safari 26+, recent Firefox) | native Linux (executor) | native macOS (developer) | native Windows x64 |
|---|---|---|---|---|
| renderer | full | full with a Vulkan GPU, else none: the simulation, the probe and the playtests without their frames | full (Metal) | full (DX12) |
| lanes | with COOP/COEP | rayon | rayon | rayon |
| audio | AudioWorklet | cpal or null | cpal | cpal |
| capture | readback | readback | readback | readback |
| determinism | yes | yes | yes | yes |

The engine targets current hardware and current browsers; old browsers,
low-end GPUs and old drivers are not a target and are not tested. Today a
browser or device without WebGPU is not supported, and a feature is built for
the supported devices at the quality it needs. The planned GL tier (§2.6) is a
separate tier, not a fallback inside the WebGPU path.

- Browsers: the current stable Chrome and Edge and the major before it,
  Safari 26+, the current stable Firefox, on WebGPU.
- Desktop: Windows 10/11 x64 on DX12 with a current NVIDIA, AMD or Intel GPU;
  macOS on Apple silicon (Metal); Linux x64 with a Vulkan GPU.
- Phones: a current iPhone in Safari. Android is not yet targeted.
- Which GPU records the reference frames and which hosts are compatibility
  checks is docs/ci.md's (§The GPU host).

Windows x64 uses the same native API (`og.exe`, `origin-engine.node`, DX12 by
default); `og check` uses the tsgo fallback there.

## 9. Branding

- **Name**: Origin Engine.
- **Prefix**: `og` everywhere a short identifier is needed: the CLI, crates
  (`og-*`), formats (`.ogt`, `.ogb`), the host ABI (`og_*`), the TypeScript
  root object. The platform uses `og-` for credentials, `og-probe` for the
  wire and `OG_*` for environment.
- **Packages**: `@origingame/engine` (with `knowledge/`), its per-platform
  native packages and `@origingame/create-game`, released as one signed
  archive per target (`docs/release.md` §1). Repository
  `originai-game/origin-engine`.
- **Artifacts**: `origin-engine.wasm`, `origin-engine.node`; telemetry and
  user-agent token `OriginEngine/<version>`; `hello.engine` reports
  `origin-engine <version>`.
- **Marks**: the boot splash is the master symbol in white on ink `#14161b`
  playing `genesis` once; loading uses `progress` with a real value; the
  "Made with Origin Engine" badge in the player shell and the exported page's
  footer. The product mark `engine` in `packages/brand` is the core ring with
  the symbol's two inner arms; its geometry lives in
  `packages/ui-svelte/src/lib/brand/product-marks.ts`, never redrawn.
- **Game UI is not brand, and brand is not game UI**: nothing of the
  platform's design language reaches a game's screens. Only the engine's
  marks, the boot splash, the loading state and the player shell use the
  platform ink and tokens.
- **Voice**: documentation, the wiki and `og` output use plain sentences, no
  exclamation marks, and every error names the fix.
- **Versioning**: semver; a game pins its engine version in `project.ogt`;
  upgrades are explicit.
- **Licence**: MIT (`LICENSE`).

## 10. Depth features

The feature areas below are referenced by their code; each section names the
modules that own it. The IDL is the contract for every signature. A type or
parameter that has a three.js name takes that name. `debug`, `expect`,
`trace`, `describe`, `save_scene` and the `og render` machinery are
development members, marked so in the IDL and absent from `publish`. Paths
and the timeline's playback are §2.7, the video producer §2.9, rooms and
lockstep §2.8 and §2.10, the semantic frame and diagnostics §3.4.

### 10.1 Runtime geometry, lines, points, debug draw, mesh raycast

- Games make geometry from attribute arrays (`og.render.bufferGeometry`,
  rewritten in place with `updateGeometry`) and from three.js's primitives,
  with three.js's parameters, counts, vertex orders and uvs. A made geometry
  is also a physics mesh under its handle.
- Procedural modelling (`crates/og-geom`: shape, extrude, lathe, sweep,
  merge, CSG) gives the same result on every host: CSG decides with exact
  predicates and generators take trigonometry from series, never libm.
  og-geom never depends on og-render. A `type model` `.ogt` builds a model
  from the same parts in the cook.
- Lines and points are one screen-space pass, so widths and dashes behave the
  same on every backend; it draws in the scene or as an overlay after post
  and before the UI, where debug draw (`og.debug`) and the debug views
  (`RenderOptions.debugView`) draw.
- `og.render.raycast` tests what is drawn, not colliders, through a BVH per
  geometry; inside a system it reads the meshes as recorded at the tick's
  start.
- Code: og-render `lines.rs`, `points.rs`, `debug.rs`, `geometry/`; og-geom;
  the `render` and `debug` IDL modules.

### 10.2 Paths and the timeline

- Paths (`og.path`): curves measured by an arc-length table so a parameter
  0..1 moves at even speed; ridden by `og.PathFollower`, `og.CameraRail` and
  timeline cameras, declared in scenes as `path` blocks.
- A `type timeline` asset is the cutscene and the edit decision list the
  video producer plays: tracks of camera, transform, animation, audio,
  music, caption (imported from `.srt` or word timestamps), event (a
  command-channel action), transition and marker keys. `og.Timeline` plays
  it after the game's systems and before og.anim and physics; its rules are
  §2.7's.
- Code: `og-timeline` (evaluation, no ECS), og-cook (timelines, captions,
  paths in scenes), the `path` and `timeline` IDL modules.

### 10.6 Mesh LOD, reflection probes, screen-space reflections

- **Mesh LOD.** The cook simplifies every model mesh into levels (glTF
  `MSFT_lod` kept) and measures each level's error in model units
  (`og cook --report` `lodErrors`). The renderer draws the coarsest level
  whose error projects under one pixel, with hysteresis, scaled by
  `og.Model.lodBias × RenderSettings.lodBias`. Every view and shadow pass
  draws the level the camera picked.
- **Reflection probes** (`og.ReflectionProbe`): box-projected cubes captured
  with the lit pipelines and prefiltered like the environment, chosen per
  instance, at most `MAX_REFLECTION_PROBES`. A `baked` probe is rendered by
  `og bake probes` into the cook cache under a key of its content, so a
  changed room is a new key, and `og doctor --project` reports a stale one;
  a `realtime` probe redraws every `interval` ticks.
- **SSR** (`PostEffects.ssr`): stochastic screen-space reflection after AMD
  FidelityFX SSSR and its denoiser (MIT, `third_party/fidelityfx-sssr`),
  on opaque surfaces of the main perspective 3D camera, over the probes and
  environment.
- **Global illumination** (`og.GlobalIllumination`): screen-space GI, DDGI
  probe GI after Wicked Engine's (MIT, `third_party/wicked-ddgi`) traced in
  software (WebGPU has no hardware ray tracing), GTAO bent normals (XeGTAO,
  MIT), and realtime light volumes. The live procedural sky drives the
  image-based light without a half-made sky ever bound.
- Code: og-cook (lod), og-render `lod.rs`, `probes.rs`, `ssr.rs`, `ssgi.rs`,
  `gi_probes.rs`, `sky_capture.rs`, `probe_grid.rs`; og-cli `bake_probes.rs`.

### 10.7 Baked lighting, irradiance volumes, volumetric fog, area lights, cookies

- **Baked lighting** (`og bake lightmaps`, og-bake): a native CPU path tracer
  with a fixed seed, so a bake with given samples is repeatable. Static
  entities get xatlas lightmap UVs storing indirect light only; each
  `og.LightVolume` gets L1 spherical-harmonic probes. The runtime uses a bake
  only while its scene and model hashes match; otherwise the scene renders
  unbaked, `og.render.bakedLighting` gives the reason and `og doctor
  --project` warns `unbaked`.
- **Projection painting** (`og bake projection`): a native CPU tool that
  textures a gray level glb from views (the trompe workflow, Apache-2.0). The
  image model stays outside the engine; the agent runs the steps and calls it
  between `prepare` and `project`. A run directory holds every step, so a run
  resumes.
- **Volumetric fog** (`og.Fog.volumetric`): a froxel grid by default or a
  per-pixel raymarch, lit by the sun and the clustered lights;
  `RenderCapabilities` reports the method and the per-sample light bound.
- **Area lights** (`og.RectLight`): linearly transformed cosines (Heitz et
  al., `third_party/ltc`), optionally shadowed within the shadowed lights'
  budget.
- **Light cookies** (`SpotLight.cookie`, `PointLight.cookie`) take layers of
  the local shadow array, no texture slot, up to
  `RenderCapabilities.maxLightCookies`; cookies past the limit are not drawn
  and the frame warns once, naming the lights.
- Code: og-render `lightmap.rs`, `volumetric.rs`, `area.rs`; `og-bake`; og-cli
  `bake_lightmaps.rs`.

### 10.8 Render targets, secondary cameras, layers, custom shaders, custom post passes

- **Render targets** (`og.render.renderTarget`): a colour texture (`ldr` or
  `hdr`) and an optional linear depth, sampled like any texture. Snapshot and
  restore keep them.
- **Views** (`og.CameraView`, `og.Mirror`): secondary cameras drawn before
  the main pass in `order` into a target or a viewport rect, each drawing
  what the main pass draws from its eye and on its layers. A mirror reflects
  the main eye through an off-axis frustum on its glass. The probe's
  `describe` lists what each view shows (§3.4).
- **Layers** are three.js masks (bit n is layer n, default 1, drawn when
  `a & b != 0`), tested by every camera and by `og.render.raycast`; shadow
  maps take every caster.
- **Full custom shaders** (`type shader`, `mode full`): own `vertex` and
  `fragment` entries over a fixed binding contract (camera, model matrix with
  skin and morphs, params, eight texture slots, standard helpers); the
  prepass, shadows and views run the user's vertex entry. The cook validates
  them with naga.
- **Custom post passes** (`type post`, `PostEffects.custom`): up to
  `RenderCapabilities.maxCustomPosts` WGSL passes after motion blur and
  before exposure; one the device refuses is skipped and reported
  (`og.render.postErrors`).
- **Crowds** (`og.CrowdAnimation`): GPU-culled, GPU-skinned members with an
  optional impostor last level; without the capability they take CPU lists
  with the same members at the same levels.
- A tier limit is reported through capabilities, never by a missing member.
- Code: og-render `targets.rs`, `views.rs`, `custom.rs`, `post_custom.rs`,
  `crowd.rs`; og-cook (shader modes, `type post`); the IDL's
  `render-targets` part.

## 11. Temporal upscalers: TAAU and FSR 3.1

Under TAA the engine resolves the jittered frame, drawn at the render scale,
to the output with `RenderSettings.upscaler`: `taau` (`taa.wgsl`, one pass
into an output-sized history) or `fsr3`, AMD FidelityFX Super Resolution
3.1's upscaler ported to WGSL (MIT, `third_party/fidelityfx/`, whose README
lists every deviation), with the order-independent transparency's coverage
as its reactive mask. FSR runs in fp32 without subgroups or `shader-f16`, so
one shader serves every host and tier; `RenderCapabilities.upscalers` names
both everywhere. Other web upscalers (pmndrs/upscaler, three.js `TAAUNode`,
Arm ASR, Snapdragon GSR 2) beat neither, so none is ported.

**The choice.** FSR 3.1 is the better picture whenever something moves and
steadier everywhere; TAAU is mostly cheaper (FSR costs about 0.65 ms to
1.05 ms more at 1080p on thin geometry), converges faster after a cut and is
as good on grass and particles. So the default, low and medium presets use
TAAU (also every native run's resolve at render scale 1), and high uses FSR
3.1; a game sets `RenderSettings.upscaler` itself at any preset. Automatic
quality keeps the upscaler while it lowers the render scale and draws TAAU
once it caps the quality at medium or low. The measurements are
`fsr::compare` and `fsr::cost` (`crates/og-tests/tests/fsr.rs`, ignored, run
by hand on a GPU); their numbers go in reports.

## 13. Alternatives considered

- **Bevy as the base.** Rejected: its API changes every release, which defeats
  the stable API the agent's knowledge depends on, its wasm builds are large
  and web multithreading is unsupported. Its crates (wgpu, naga, glam) are
  used directly.
- **Keep Godot, improve the tooling.** The headless footprint, the
  editor-centric data model and GDScript remain.
- **Three.js with an imposed structure.** Leaves JavaScript as the per-frame
  engine and the iframe as the isolation.
- **A C engine like HZ.** Rust gives the same wasm output with memory safety.
- **Game code in Rust.** Rejected for compile latency, toolchain size and the
  author's loop. On hosts without a JIT (iOS) game code ships as bytecode,
  and hot paths move into engine primitives (`docs/roadmap.md` §Direction).
- **A declarative UI language.** Rejected: it would be a third text format;
  UI structure is Svelte templates and its look standard CSS (`docs/ui.md`).

## 14. Sources

Observed shapes, not code: the HZ engine's AI-native workflow (command
channel, evidence loop, wiki-as-map, reflection to blobs, one build tool);
wgpu, naga, rapier, meshoptimizer, KTX2/Basis, QOA, MSDF, Clay layout; the
platform's probe protocol 5, its play-surface and agent-system decisions, and
the platform's design language for the player shell and the marks only (§9).
