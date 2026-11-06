# Origin Engine — design

Normative: the engine's current design. Open work is in `docs/roadmap.md`;
the UI is `docs/ui.md`.

## 1. Problem and non-goals

**Problem.** OriginGame produces games through an agent that must write, run,
observe and correct a game in minutes. Neither Godot nor Three.js was
designed for that author. Godot is editor-centric: scene files, import
caches to prewarm, a 40 MB Web export, GDScript, and a 2.5 GB executor image
to run it headless. Three.js is a scene-graph library, not an engine: every
game reinvents structure, and a JavaScript game can only be isolated inside
an iframe, which costs the play surface its viewport, gestures, audio and a
renderer process. The engine the agent needs does not exist, so we build it.

**Goals.**

1. A full mid-size 3D engine (and 2D as a subset): PBR rendering with shadows,
   IBL and post, skeletal animation and IK, rigid-body physics and character
   control, navigation, spatial audio, particles, terrain, decals, text and a
   UI toolkit. Delivered to the Web only; native builds exist for executors.
2. AI-native from the first frame: a command channel, a probe, world dumps,
   deterministic replay, frame capture and parameter sweeps are runtime
   features in every build.
3. Game code in TypeScript against a stable, documented API; no compiler wait
   and no Rust toolchain in the author's loop.
4. Performance that justifies replacing Three.js: the heavy per-frame work
   (culling, draw submission, animation, physics, particles, navigation) runs
   in WebAssembly on WebGPU, not in JavaScript. The claim is "batched,
   instanced and culled by default, so the slow version cannot be written".
5. One play surface without iframes.

**Non-goals.** Native store exports (APK, iOS apps). A visual editor as the
primary authoring surface (an inspector and gizmos exist for humans; authoring
is code and text). Scripting languages other than TypeScript. Console
platforms. Ray tracing. Networking engines beyond the platform SDK's rooms.

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

`og check` type-checks game code with tsrs (the Rust port of TypeScript 7),
a pinned binary `og` runs as a child process and never links: from
`OG_TSRS`, the `@tsrs/<platform>` package beside the engine, or `PATH`
(`scripts/install-tools.sh`); where tsrs has no package (Windows, Intel
macOS) it falls back to tsgo through `bun x` and says so. Diagnostics are
values: `--json` prints one `check` event, `{file, line, col, code,
message}` per diagnostic, on the same channel as the frames and the dumps.
`og run`, `og test` and `og build` check before they bundle and stop on a
type error (`--no-check` skips it; `og build --publish` refuses
`--no-check`); `og dev` checks each rebuild beside the bundle and reports
it on the `reload` event without refusing the reload. A checker that
crashes or prints nothing readable is a failure, never a clean check. tsrs
is the repository's checker too (`packages/engine`'s `typecheck`); tsgo is
the conformance comparison (`check-conformance`, docs/ci.md) and the
fallback.

### 2.2 Threads

- **Main thread**: the shell only. Collects input into a SharedArrayBuffer
  ring, owns the AudioContext (unlocked once on the first gesture), the PWA,
  platform UI and the creator overlay. Never runs game code.
- **Engine Worker**: the wasm instance, the renderer on an OffscreenCanvas,
  the game bundle. The tick and the frame run here; a slow frame never stalls
  the shell.
- **Lanes**: parallel systems through `wasm-bindgen-rayon` on a pool of
  Workers sharing the wasm memory. Requires cross-origin isolation
  (COOP/COEP) on the play page, which the platform declares per
  publication. Render recording stays on the engine Worker: wgpu's web
  backend objects are not shareable across threads. Without isolation the
  engine runs single-threaded; the API is the same. A lane never changes a
  result: helpers return in item order, and a library that splits across
  the pool itself (rapier's `parallel` build, bitwise identical on any
  number of threads) runs there only from a grain of work for every pool
  thread, on the engine thread below it.
- **Native host**: the same split (shell = `og`, engine on its own thread,
  lanes via rayon).

### 2.3 Data model

- **World**: an archetype ECS. Entities are `u64` ids with a generation;
  components are plain data declared with `#[derive(Reflect)]` in Rust (no
  `Component` marker: any reflected type can be a component, resource or
  event) or `defineComponent({...})` in TypeScript (which registers the same
  reflected layout at load time). Component storage is SoA in wasm memory.
- **Typed views**: a TypeScript system receives `Float32Array`/`Uint32Array`
  views over the component columns of each matched archetype plus the entity
  list. One call per archetype per system per tick, never one call per
  entity. Views are invalidated by structural changes; the API hands them out
  per tick and refuses stale ones (a stale view throws with the tick it
  belonged to).
- **Resources**: singletons (time, input, camera, tuning), also reflected.
- **Reflection**: every component and resource has a schema (fields, types,
  ranges, labels, units, `tunable` flags, docs). The schema is exported as
  JSON (`og schema`) and is what the inspector, Tune, the probe's `bounds`,
  scene text and the generated wiki read.
- **Handles**: assets, meshes, materials, sounds and scenes are generational
  handles; a dead handle is an error value, never a crash.

### 2.4 Time and determinism

- Fixed tick (default 60 Hz, per project), accumulator with a cap; render
  interpolates between the last two ticks (`RenderSettings.interpolate`,
  on by default: a display frame blends transforms, the camera, skinned
  joints and world-anchored UI by the accumulator's fraction; a frame on a
  tick boundary and every probe capture draw the tick, so reference frames
  are the ticks'; og-runtime's `interpolate.rs`). Game code never reads
  wall-clock time: `Time` is a resource with `tick`, `dt`, `elapsed`.
- Randomness is `Rng` resources seeded from the project seed and the entity
  or system that owns them; the sequence is identical on wasm and native
  (`rng.ts` and `rng.rs` share test vectors).
- Snapshot and restore of the whole world (components, resources, rng
  states, pending events) to a blob; `sweep`, hot reload and `feel_compare`
  are built on it, and so are a game's saves (`world.snapshot` and
  `world.restore`, in every build: the blob as text for the platform store,
  restored with the session kept).
- Save/load delivers `og.WorldRestored { from, tick }` on the first tick
  after `world.restore`. Before that tick, the restored world hashes as
  the saved world did. The notification is observable simulation state:
  it is included in that tick's events, snapshot and `world_hash`. A replay
  without a restore therefore has a different hash on the first resumed
  tick, even when no system handles the event. With no resulting state
  changes, subsequent ticks match again. Rollback's internal rewind does
  not emit this notification: it repeats history rather than loading a save.
  Save/load replay tests compare equal notification histories, or compare
  the first resumed snapshots with only `og.WorldRestored` removed and
  compare subsequent hashes normally. They must not ignore all differences
  on that tick. `og-runtime`'s `restored_event` tests check the event's
  lifetime and byte-for-byte parity of the remaining snapshot state.
- Floating point: simulation uses `f32` with the same operations on both
  hosts; the engine ships its own `sin`/`cos`/`atan2`/`exp` so results do not
  depend on the host's `Math`. Physics (rapier) is built with its
  `enhanced-determinism` feature.
- Large worlds: a world that names `og.WorldOrigin` keeps every place near
  its zero by shifting as a tick starts (og-runtime's `origin.rs`, a
  before-tick hook ahead of cell streaming), when its focus
  (`WorldOrigin.focus`, else the first `og.StreamAnchor`, else
  `og.Camera.position`; a net session refuses the camera) is farther than
  `step` from the zero along an axis of the plane, by the multiple of
  `step` nearest the focus. The shift is world state decided from world
  state, so replays, restores, rollbacks and lockstep peers shift on the
  same tick; the zero is kept in f64 (`WorldOrigin.x/y/z`) and
  `og.OriginShifted` carries the delta for places a game keeps itself.
  What moves is read from the schema: a field marked `worldspace world` (a
  place in the world), `root` (a place while its entity has no parent),
  `hook` (a place under a condition its module's state block knows, such
  as og.Joint's `otherAnchor` while `other` is 0) or `none`, with its axes,
  in the IDL and the Reflect derive; og-idl refuses a `Vector2`/`Vector3`
  record field whose name reads as a place (position, point, target,
  centre, anchor, origin, pivot, focus, eye) without one.
  `og_ecs::World::shift` moves the marked columns, their change shadows
  with them (a shift is not a change), resources and queued events; every
  `StateBlock` implements `shift` (required): rapier's bodies and
  parentless colliders move in place, keeping contacts, warm starts and
  islands, with what slept put back to sleep; navigation keeps a per-nav
  frame offset, the og.cells block its subtrees, and terrain brushes stay
  in the logged frame on purpose. The render origin (§2.6) is laid on the
  absolute world's 8192 m grid, so a shift never moves it. 2D worlds shift
  too, on `plane xy`. A power-of-two `step` (default 2048) keeps every
  shift exact in f32; another step rounds each place once per shift. A
  world that never names it keeps its hash and frames.
- The presentation layer is outside this: what is computed for display
  only, such as the motion model's poses (§2.7), may differ between
  devices, precisions and runs. It never writes a component the simulation
  reads, is not in a snapshot or `world_hash`, and game code does not
  branch on it; the simulation stays bit-identical on every host.

### 2.5 Scenes and assets

- **`.ogt`** is the one text format: a line-based `type`/`version` header, a
  separator, then `key value` lines with nested blocks. Scenes, materials,
  tuning, import settings and the project manifest (`project.ogt`, whose
  `budgets { }` hold the asset budgets `og doctor --project` checks) all
  use it. It diffs line by line in git; the agent's checkpoints are git
  commits.
- **`.ogb`** is the one cooked blob: type hash, schema version, cook stamp,
  then the reflected struct serialised with its asset payloads (meshes as
  meshopt-encoded vertex streams, textures as KTX2 Basis Universal, audio as
  QOA, fonts as MSDF atlases, scenes as component tables). Blobs load by
  copy-free view where alignment allows.
- **Cook** (`og cook`): incremental, hash-stamped, runs glTF import
  (meshoptimizer simplification and vertex cache ordering, LOD generation,
  tangents), texture compression, audio transcode, MSDF generation, scene
  compilation. Source assets live in `assets/`, cooked output in `cooked/`;
  the runtime reads only `cooked/`. Colour textures are ETC1S unless
  project.ogt's `cook { color_textures uastc }` says otherwise (four times
  the bytes, no 4×4 block colour loss); `publish { color_textures etc1s }`
  overrides it for the web export's cook (`og build`, `og run --device`).
- Reusable production assets come through the platform's asset MCP; the
  cook consumes what the agent put in `assets/`.

#### Uncooked loads

Content a player makes or a generator writes at run time cannot go through
`og cook`, so the runtime reads two formats from bytes, uncooked:
`og.load.imageData` (PNG, JPEG) and `og.load.gltfData` (a `.glb`, or a
`.gltf` whose buffers and images are `data:` URIs). Both answer a pending
handle that turns ready or failed like any load, and the result is drawn as
a cooked texture or model is (`og.Sprite`, material maps, `world.instantiate`,
`og.Model`, `og.load.gltfInfo`).

- **Budgets before decoding.** `LoadBudget` (bytes 32 MiB, triangles
  500,000, textureSize 4096 and never past 8192, frameMs 2) is checked at
  the call from the headers alone (the file's size, a PNG's IHDR or a
  JPEG's frame header, a glTF's JSON and each embedded image's header). A
  load over a limit is refused then, naming the limit, the value and the
  fix; nothing is allocated for it.
- **The reader** (og-bundle `runtime::UncookedGltf`, serde_json, no `gltf`
  crate in the wasm) builds the same `CookedModel` the cook's blob decodes
  to, joining the renderer through the cooked models' insert path. It reads
  the metallic-roughness materials and their maps, `KHR_materials_unlit`
  and `_emissive_strength`, nodes and scenes, vertex colours and tangents;
  a colour texture magnified nearest is pixel art. Absent: levels of
  detail, cooked raycast BVHs (a raycast builds one when first needed),
  meshopt-coded streams, compressed textures. Skins and animations are
  skipped and counted. Refused by name: Draco and meshopt compression,
  Basis, WebP and AVIF textures, sparse accessors, a buffer or image in
  another file, any other required extension.
- **Off the engine's thread.** Images decode in the browser on the web
  (`createImageBitmap` and an `OffscreenCanvas` in the engine's Worker,
  behind a host hook; no image decoder is in the wasm) and on a worker
  thread natively (the `image` crate). A glTF's primitives are converted at
  tick boundaries, one at a time within `frameMs` on the web (at least one
  a frame), all at once natively.
- **Completion is host input.** On the web each load is put in place at a
  tick boundary and journaled as `{ request, tick, ok }`
  (`og.load.completions`). `og replay --record` keeps the journal in its
  trace; `--compare` and `og.load.replayCompletions` hold each load until
  its recorded tick, and a recorded failure fails. Natively a load is in
  place at the boundary after its call. An asset's status is not world
  state. In a lockstep or rollback room a completion stays local: a game
  sends the bytes with `og.platform.roomSend` (at most 64 KiB a message),
  each peer loads them, and the shared simulation uses the content only
  from a tick the peers agree on through their inputs (`og.net.submit`),
  never from `og.load.status`.
- **Release.** `og.load.release` frees an uncooked asset. A pending load is
  dropped and never journaled. A ready one's textures and meshes are emptied
  under their handles, so whatever still names them draws nothing; their
  GPU memory and the scene's CPU copies go, with the model's draw parts and
  cached BVHs, and the handle answers failed. Physics shapes built from its
  geometries are world state and stay. `dump_memory` (§3.3) shows both
  sides.

#### Prefabs

A prefab is a reusable entity tree authored once and placed many times. An
instance stores only what differs from its prefab, so a prefab edit reaches
every instance on the next cook and an instance's edits survive it.

**The asset.** `assets/**/*.ogt` of `type prefab`, version 1: exactly one
top-level `entity "<root>" { ... }` in the scene grammar (components,
`model`, child `entity` blocks), and nothing else (no `resource`, `path` or
`stream`). Entities inside a prefab are addressed by their **node path**,
the names from the root down joined by `/` (`""` is the root, `lid/hinge` a
grandchild); names are unique among siblings, which the cook checks.

**Placing one (nesting).** Anywhere an entity can appear, in a scene or
another prefab, an entity may be an instance:

```text
entity "crate1" {
  prefab "prefabs/crate.ogt"        # its tree is the prefab's
  Transform { position 4 0 2 }      # an override of the root
  override "lid" {                  # an override of a descendant
    Health { hp 40 }                # field lines: only those that differ
  }
  remove "lid/sticker"              # a descendant this instance drops
  entity "loot" { ... }             # an entity added under the root
}
```

Component blocks and `override "<node>"` blocks list only fields that
differ; a component the prefab node lacks is added. `remove "<node>"` drops
a node and its subtree; `remove "<node>" Health` drops one component.
Instances nest to any depth; a cycle (through instances or a base) is
refused naming the chain.

**Variants.** A prefab whose header body starts with `base
"prefabs/goblin.ogt"` is a variant: its one `entity` block is an instance of
the base in the instance grammar above. Variants chain; the root keeps the
variant's own entity name.

**Cooking.** The cook flattens depth-first in text order: an instance
becomes its prefab's flattened tree, then the variant chain's deltas from
the base outwards, then the instance's own, each in text order. The scene
blob is unchanged in shape, so spawning, hooks and the world hash treat a
flattened prefab exactly as the same entities written by hand. Added beside
it:

- a `prefabs` table: per entity, the prefab asset id and node path it came
  from, and the index of its instance root (or none);
- the stamp: a scene's (and a prefab's) cook stamp includes the source hash
  of every prefab it reaches, transitively. Prefabs cook to a blob of their
  own too, so `og.load.prefab` loads one for `world.instantiate`.

A field the instance does not override follows the prefab; one it does keeps
its value. An **orphaned** override or `remove` (its node no longer
exists), or an overridden field the prefab's component no longer has, stays
in the text so a rename back restores it. A dev cook warns; `og prefab
check` exits non-zero while any exists; a publish cook refuses it, naming
the instance, the node and the field. Everything else is refused as for a
scene.

**Names.** An instance's root takes the instance entity's name; each
descendant is named `<root name>/<node path>` (`crate1/lid/hinge`), so a
placed tree is addressable by the probe, `world.find` and the command
channel. A generated name that collides with another fails the cook, naming
both. An unnamed instance root leaves its tree unnamed.

**At run time.** Each entity spawned from a prefab carries
`og.PrefabLink { prefab: Asset, node: u32, root: Entity }`, written from the
`prefabs` table (`node` indexes the prefab's flattened table). It is
ordinary world state: snapshots, restores and the hash carry it, so a game
that places prefabs hashes differently from the same entities written by
hand, and stably across a re-cook that changes nothing it spawns. A game
without prefabs has no such entity. API: `og.load.prefab(path)` (an
`Asset`), `world.instantiate(asset, SpawnDesc)` (the desc's root is the
instance root), and `world.prefabOf(entity)` (`Option<PrefabLink>`).

**Write-back and apply-back (dev builds).** `save_scene` keeps instances as
instances: a tuned field of a linked entity is written as an override line
in its instance's block, only when it differs from the prefab's cooked
value; a field tuned back to the prefab's value loses its override line.
`apply_prefab:<entity>[:<Component>[.<field>]]` (a command-channel action;
no RPC member) writes the instance's overrides on that node (all, a
component's, or one field) into the prefab's `.ogt` and removes them from
the instance's text, so the scene cooks to the same world and every other
instance takes the new value. On a variant it writes into the variant's
text, and `:base` writes through to the base. A run-time value that was
never an override (set through `set`) is applied the same way.
`revert_prefab:<entity>[:<Component>[.<field>]]` removes overrides and sets
the live values back to the prefab's.

**Tools.**

- `og prefab check [project]`: cycles, orphaned overrides and removes,
  overridden fields the prefab no longer has, sibling name clashes.
- `og prefab show <prefab> [--scene <id> --instance <name>]`: the flattened
  tree, each field with where its value comes from.
- `og prefab extract <scene> <entity> <prefab path>`: an authored subtree
  becomes a new prefab and the scene's block an instance of it with no
  overrides.
- Probe: `dump_entity` has `prefab` (asset, node, root, overridden fields);
  `apply_prefab`, `revert_prefab`; `describe` lists instances by prefab.
  Additive to protocol 5.

**Determinism.** Flattening is a pure function of the texts, so both hosts
spawn the same entities in the same order with the same values.

#### Sub-scenes and world cells

Streaming groups (§2.6's assets, IDL `stream`) move what a frame draws and
plays; nothing a tick reads depends on them. World cells move entities,
which a tick does read, so what is loaded at a tick is a function of world
state alone, never of I/O timing.

**Sub-scenes.** In an entity block, `scene "<id>"` places another scene
there. It is not flattened: it stays its own blob, named in the placing
scene's optional `subscenes` table, so editing it re-cooks it alone; a
sub-scene that reaches the placing scene again is refused naming the chain.
Without a cell, the runtime spawns it under the entity.

**Cells.** An entity with a sub-scene and `og.StreamCell { scene, size,
load, unload, pin, loaded }` is a cell. As each tick starts (the probe's
before-tick hook, so replays, re-simulations and restored worlds run it,
before any system), the engine decides from world state whether each cell
is loaded: an `og.StreamAnchor { reach }` within `load` of the box (the
cell's world position ± half `size`, turned with it), kept until every
anchor is past `unload`, or `pin`. Without anchors the `og.Camera`
resource's position anchors, refused while `og.Lockstep.enabled`. A load
spawns the sub-scene: its top-level entities are roots at the cell's pose
after their own, every spawned entity carries `og.CellMember { cell }`; an
unload despawns every member and its descendants (a game keeps one by
removing the member). Each change writes `loaded` and sends
`og.CellEvent { cell, loaded, entities }` in the same tick, and add hooks
and `added` filters see the entities arrive.

**Order.** Cells are visited by entity (slot, then generation); a sub-scene
spawns in its blob's order; an unload despawns children first, by entity.
The same world allocates the same entity ids on both hosts, in a replay and
after a restore (og-runtime's `tests/cells.rs`).

**Snapshots and saves.** `loaded` is a component field and a loaded cell's
entities are ordinary entities, so a snapshot, a save, the hash and a
rollback carry exactly which cells are loaded and what is in them.
`StreamCell.scene` is saved as its handle number: a session loads the
scenes that place the cells before restoring. The four records are lazy: a
world without cells keeps its type table, saves and hash.

**Persisted cells and the spawn budget.** A cell with `persist` keeps what
happened in it: as it unloads, its members and their descendants (parents
first) are kept as rows (saved id, name, every component with its layout
hash) in the `og.cells` state block. As it loads again the rows spawn
instead of the sub-scene, entity fields naming a kept row point at its new
entity, and the rows are dropped; rows from a build whose components have
another layout are refused with a warning and the cell loads as authored.
`budget` caps the entities a load spawns per tick; the load in progress is
in the same block, so a restore taken mid-load goes on. `loaded` turns true
and `og.CellEvent` is sent the tick the last entity spawns; a cell no longer
wanted before then despawns what it spawned and sends nothing.

**A cell that is not ready.** A sub-scene's blob is in the boot bundle
unless the sub-scene declares a cell group, `stream "<name>" { cell; asset
... }` (at most one, only in a placed scene, with no distance or volume):
the cook packs the blob into that group, and the runtime's handle waits
(`Payload::SceneWaiting`) until the pack has landed. A cell group is wanted
while one of its cells is loaded, loading or pinned, or an anchor 2 s
further along at its `StreamAnchor.maxSpeed` would be within the cell's
load distance (plus its reach). A tick that would load a cell (from its
sub-scene, not kept rows) whose blob has not landed is held by a gate
chained before og.net's, never skipped or reordered; the gate asks for the
group first in admission. Natively a read completes at once, so a native
run never holds. On the web a held tick keeps frames drawing,
`dump_stream`'s `hold` names the tick, the cell, the group and its state,
and the log records each hold (`holds`: count, total, longest). After a
rollback's rewind the gate asks again. og.StreamEvent is world state: the
ticks that deliver a group's events hash them. Its models, textures and
sounds draw stand-ins until resident; physics reads colliders only from
components and proxies.

**Baked colliders.** A model cooked with `model { collider true }` carries
each primitive's triangle-mesh shape baked over LOD 0 exactly as the
runtime decodes the blob (the proxy's level for a streamed model), in its
`colliders` section; physics takes it as the model loads, and a
SHAPE_TRIMESH collider at scale 1 naming the geometry uses it while the
mesh is bit-for-bit the one baked, so nothing a tick computes changes. A
scaled collider, a convex hull and the 2D profile build at run time.
Opt-in: about 90 bytes a triangle.

**Texture mips.** A texture cooked with `texture { stream true }` (or a
model's `model { stream_textures true }`) ships twice from one encode:
`<id>`, its mip tail (the levels from the first at most 256 px on its
longest side), in the boot bundle, and `<id>#mips`, the whole chain,
fetched on demand (never preloaded; a publish build caches it lazily). A
texture within 256 px, pixel art and one in a streaming group cook whole.
Before each frame (og-runtime's `stream/mips.rs`) each streamed texture's
footprint is the most pixels any instance drawing it covers; one past its
tail's longest side is wanted whole, and stays wanted 120 frames after it
was last. The wanted ones are admitted by footprint over tail (then id)
within a mip budget of 128 MiB, separate from the groups'; read after the
sound chunks and packs, transcoded and swapped in under the same handle;
one no longer admitted goes back to its tail. `texture_size` reports the
whole chain throughout. Presentation only. The probe's
`mip_stream:resident` shows every streamed texture whole before each frame
(natively at once, so captures are whole); `mip_stream:stream[:<MiB>]`
streams again, and `dump_stream`'s `mips` lists the mode, the budget, use
and each texture's state and footprint.

### 2.6 Renderer

wgpu with WGSL shaders cooked per backend (naga). On the web the renderer
is WebGPU only (§8): compute is available on every supported device, and a
feature is designed for that tier. The CPU culling and CPU particle paths
remain options on every device (`Culling::Cpu`, small frames under
`Culling::Auto`, `ParticleSimulation::Cpu`). The engine is GPU only on
every host: a player's device without a GPU is not supported, and no
feature adds a CPU fallback for one. Developer tooling runs without a GPU:
the logic tests and the probe's CPU description of a frame.

- Order-independent transparency for the main camera's blended surfaces
  (`oit.rs`, `RenderOptions.transparency`): weighted blended (McGuire and
  Bavoil 2013) with each pixel's nearest layer exact (hybrid transparency,
  Maule et al. 2013; marked by an alpha-only depth pass into the same
  arena) by default; per-pixel lists as the high tier, one packed atomic
  arena at group 0 binding 14 with a fixed node budget a pixel, sized
  against the device's limits, every fragment also weighted and a pixel
  past its budget resolved whole as weighted and counted (stats,
  `dump_render`). Resolved per MSAA sample before TAA, whose history
  weight the revealage lowers (a reactive mask). Additive effects, 2D
  sprites, text, the UI and secondary cameras keep their order.
- Forward+ (clustered), with a depth-normal prepass only when an effect
  reads depth or normals; PBR metallic-roughness (glTF-compatible), normal,
  emissive, occlusion, and three.js's physical lobes (clearcoat, sheen,
  transmission, iridescence, anisotropy).
- Cascaded shadow maps for one directional light (opt-in PCSS soft shadows
  and blending across cascade splits); cube faces or two dual-paraboloid
  hemispheres for up to 8 point/spot lights, the brightest at the camera
  first (cube by default; `RenderOptions.pointShadows`). A local light's
  layer holds the midpoint of its casters' near and far sides (Woo's
  midpoint shadow map, one pass into two channels: `shadow_target.rs`).
  The cascades store far sides.
- IBL from an environment map prefiltered at cook time, as three.js's
  scene.environment and background: a blurred background from the
  prefiltered levels (`backgroundBlurriness`), a grounded dome
  (`groundHeight`), and a blend towards a second map (`blendTo`,
  `blendWeight`, drawn in 128 steps, driven from the tick) mixing every
  prefiltered level and the harmonics; uncooked `.hdr` (and natively
  `.exr`, `RenderCapabilities.exrDecode`) loaded at run time by
  `og.load.environmentData`, decoded off the engine thread or in budgeted
  rows and GGX-prefiltered on the GPU (the cook's CPU `bake` stays for
  cooked maps and tools); sky model with exposure; height fog, lit as a
  medium when volumetric; baked lightmaps and light volumes (below).
- Post: bloom, tonemap (ACES, AgX or Khronos PBR Neutral), colour grading
  LUT (a photo editor's adjustments baked into it on the CPU), lens
  distortion, zoom, spin and directional screen blurs, SSAO (GTAO),
  antialiasing: TAA by default (optional Catmull-Rom history and
  sharpening), 4× MSAA or FXAA as choices, FXAA where the HDR target cannot
  be multisampled. TAA reprojects along the prepass's motion vectors where
  something moved besides the camera (instances with
  `previous_transforms`, skinned meshes with `previous_joint_palettes`,
  filled by og-runtime from the frame drawn before; under GPU culling
  instances move with the camera only). Below render scale 1 it is
  temporal upscaling (TAAU): the scene is drawn at the render scale with a
  Halton jitter over 8 phases per output pixel a scene pixel covers (up to
  32), TAA writes an output-sized history from the sample nearest each
  output pixel, weighted by its distance, and every later pass runs at the
  output's size; a change of scale below 1 keeps the history. The resolve
  is a quality field (`RenderSettings.upscaler`): TAAU, or AMD FidelityFX
  Super Resolution 3.1's upscaler ported to WGSL in fp32 without subgroups
  (MIT, `third_party/fidelityfx/`), which takes the `taa` pass's place with
  its eight compute passes (depth-reconstructed disocclusion, thin-feature
  locks, luma instability, Lanczos accumulation, the OIT coverage as its
  reactive mask) and its RCAS in `taa-sharpen`'s; its state is sized to the
  output. §11 compares the two and gives the presets'
  choice.
- What a frame may cost is data the renderer reads every frame:
  RenderSettings' quality fields (rect-light shadows, the number of shadowed
  lights, the shadow maps' scale, volumetric fog, soft particles, SSR, AO
  and its resolution, the SSR and volumetric fog step scales, MSAA samples,
  the upscaler), og-render's `RenderQuality`. A graphics preset
  (`GraphicsQuality`: low, medium, high) is a row of them that
  `og.render.applyQuality` writes and `RenderSettings.quality` records; no
  preset state lies behind the fields. Medium is the defaults, the full
  frame. High: shadow maps at twice their side, AO at full resolution, twice
  the SSR steps and the volumetric fog's slices. The device caps
  `maxShadowedLights` and `maxMsaaSamples` (1 where the renderer resolved
  TAA, FXAA or none) and says so in RenderCapabilities; the fields keep the
  row's value.
- Stylisation, as data a look composes (`og.Stylise`, `og.MaterialStyle`, a
  `type look` asset): a logical pixel grid scaled up by whole numbers with
  the camera and sprites snapped to it, posterise and palette quantisation
  through ordered or blue-noise dither, halftone, cross-hatching, ink lines,
  painterly (Kuwahara) smoothing with pigment pooling, bleed and a canvas,
  a watercolour wash (Bousseau's pigment density from the paper's grain and
  a turbulence, rims where painted regions meet, dry brush, wobble), flat
  depth bands, retro-3D vertex snapping, affine texturing and coarse depth,
  and scene-wide material defaults (toon, rim, outlines thinning with
  distance, tinted shadows, flat facets); every primitive off at zero and
  then free.
- Skinning on the GPU (storage buffers).
- Instancing by default: draws are batched per material and mesh; the
  TypeScript API has no per-object draw calls to misuse.
- Particles: GPU simulation (CPU as an option); one emitter asset format.
  Collision against the depth buffer in compute, against the scene's
  colliders (ground, boxes, spheres, terrain height fields) on the CPU.
- Terrain: clipmap heightfield with splat materials, its field read from a
  heightmap or made by the cook (`generate`: fractal noise, an island or
  ridge mask, hydraulic and thermal erosion, terraces, a sea with its
  shore, rivers along the routed flow, biome rules writing the control
  map), with IEEE-exact arithmetic and the generator's own PCG32 so every
  host makes the same field; both routes write the same tile pyramid, and
  `TerrainInfo.seaLevel` places the water. Detail meshes scattered over it
  sway in `og.Wind` (drawn from the tick, never read by the simulation).
  Decals: deferred-style projection in the forward pass.
- Text: MSDF for UI and world text, kerned from the font's GPOS pairs; CJK
  coverage through the platform's subset fonts, and characters outside a
  cooked charset rendered at run time from the font's outlines (`font {
  runtime true }`). A pixel font (`font { pixel 12 }`, its pixels to the
  em) is cooked as its cells on that grid, sampled at the cells' centres
  and traced into square contours before the field, its advances and
  kerning in whole cells, and og-ui sets it at whole multiples of the grid
  only (run-time glyphs take the same cells). UI paragraph layout uses
  Parley with HarfRust and Skrifa behind og-ui's `shaping` feature: UAX #9
  bidi, UAX #14 line breaks and dictionary breaks for complex scripts. Its
  font collection contains cooked faces only; system font discovery is
  off. World text faces the camera as a billboard when asked
  (`TextDesc.facing`), turned as one panel about its aligned point and
  sorted among the transparent meshes.
- Capture: a readback path from the render graph, used by the probe's
  `capture` on both hosts; on the web it reads the OffscreenCanvas target,
  so no `preserveDrawingBuffer`.

**Meshlets** (after three.js#35191's `MeshletDrawingPipeline` and
`VisibilityBufferDrawingPipeline`; `meshlet.rs`). The cook cuts every level
of every model primitive into clusters of at most 64 triangles
(meshoptimizer's `buildMeshlets`, up to 192 distinct vertices) and reorders
the level's draw indices so each cluster is a contiguous range. The model
blob keeps that stream in `cluster_indices`, separate from `indices`: the
simulation mesh, its vertex-fetch order and its raycast BVH keep the
pre-meshlet order, because reordering a collision mesh changes ray hits
and contacts, and a rendering optimisation must not change the simulation.
Both streams use the same vertices and LOD ranges. Each cluster keeps its
bounding sphere (widened to its farthest vertex) and its normal cone (the
`clusters` table, each `lods` row naming its range). The levels are the
ones per-instance LOD uses (`lod.rs`), not a cluster DAG: one level is
chosen for a whole instance, from the error projected at its nearest point,
so one huge mesh draws its nearest part's detail everywhere and only
cluster culling saves work there; large assets are best authored in parts.
Every cluster row carries its own error, its parents' error, the sphere
both are projected from and its group, and a cluster is drawn when its
error projects under a pixel and its parents' does not
(`meshlet::selected`); the cook's per-level chain writes those fields so
that exactly the level `lod::select` picks is drawn.

Per frame (`meshlet_gpu.rs`, `meshlet_cull.wgsl`), after GPU culling
(`cull_gpu.rs`) has culled and compacted a clustered batch's instances, one
compute pass runs a workgroup per instance: it tests the instance's sphere
against the frustum and the depth pyramid, picks its level by screen-space
error (`lod::select` without hysteresis, with `lodBias`) unless its caller
picked it (`RenderScene::instance_levels`; og-runtime picks one level per
model by its parts' largest error, its own `lodBias` and hysteresis, so both
paths draw the same picture and every view and shadow pass draws that
level), then tests each of the level's clusters: its sphere against the
frustum, its normal cone against the eye (not for two-sided materials,
mirroring or uneven scale, or orthographic cameras), and its sphere against
a depth pyramid in two phases (`vis` keeps one bit per cluster slot across
frames): the early phase takes the clusters visible last frame that pass
frustum and cone and draws those of occluding materials into GPU culling's
occluder depth; the pyramid is rebuilt from it; the late phase tests every
cluster against it, records the visible ones for the next frame, and adds
those the early phase did not draw. The pyramid holds only depth drawn at
this frame's camera, so occlusion never removes a visible pixel. A frame
without the pyramid draws what frustum and cone keep in one phase. Visible
cluster slots are compacted into work items (instance, cluster) in row,
instance, then cluster order (`meshlet_compact.wgsl`): a blocked prefix
scan counts visible and software-eligible slots, the first 65,536 eligible
slots enter the software list and the overflow stays on the hardware list
in the same stable order. Both lists are rebuilt after each cull phase, so
neither draw order nor equal-depth software triangle IDs depend on
workgroup scheduling. Each batch is drawn once by `draw_indirect`, 64 × 3 vertices per visible cluster, by
vertex pulling (`meshlet.wgsl`: no index or vertex buffer bound; unused
triangles collapse), in the prepass and the lit pass. Under TAA with object
motion GPU culling uploads the sorted instances' previous transforms
(`cull_gpu.rs` `previous`) and compacts them with its survivors, so every
path gives TAA and motion blur each object's own motion. `debugView lod`
tints each cluster by its level. The path takes the camera pass's opaque
batches of clustered meshes under GPU culling (`Culling.auto` turns GPU
culling on when such a batch exists); skinned, morphed, lightmapped or
compact meshes, custom shaders, transparent and physical materials, texture
arrays, runtime geometry without clusters, object-space stylise and
`Culling.cpu` keep the per-instance path (`meshlet_gpu::takes`;
`RenderScene.meshlets` turns it off). Shadow maps go through the same
dispatch: each cascade and local light face is a view whose rows test the
clustered casters against that pass's planes alone (no cone, no pyramid),
at the level the camera picked, drawn by vertex pulling in the shadow
pipeline's state (`meshlet_shadow.wgsl`; alpha-tested and dissolving
casters with their material). Planar reflections, views and reflection
probe faces are views of the same dispatch, each with its planes, its eye
and its pixels per unit (half its height times the length of its
view-projection's clip y row), so each picks its own levels and culls by
the cone from its own eye; only the camera tests the pyramid. A view whose
work would pass the device's storage binding limit, or a view or probe face
with a depth prepass, keeps the per-instance path.

The visibility buffer (`meshlet_gpu/vis.rs`, `meshlet_raster.wgsl`) takes
the camera's small clusters: of an occluder batch (opaque, no alpha cut,
dissolve or material shader), under a perspective camera
(`RenderScene::visibility_buffer`, on by default), a cluster wholly past the
near plane whose sphere covers at most 24 pixels across (`meshlet::soft`)
goes to a soft list. One workgroup per soft cluster and one thread per
triangle rasterise it in compute, in two passes because WebGPU has no 64-bit
atomics: an atomic max of the depth word, then of the id word (cluster slot
and triangle) where the depth is the pixel's. Vertices snap to 1/256 of a
pixel, and the top-left rule makes shared edges cover each pixel centre
once (`meshlet::covers`). A setup pass, one thread per pixel, rebuilds the
seen triangle's attributes with perspective-correct barycentrics
(`meshlet::barycentrics`) into a record, with each pixel's uv and
world-position differences to the next pixel right and down on its
triangle's plane. One full-screen resolve per soft batch, in the camera's
lit pass and its prepass, shades that batch's pixels with the same
`lit_fragment` and `prepass_targets` as the hardware path, writing the
rasteriser's depth; its pipelines set `pbr_fs.wgsl`'s `ANALYTIC`, sampling
the maps with those gradients (`textureSampleGrad`). Retro 3D's vertex path
is repeated in the rasteriser (`meshlet::retro_clip`), and with affine
texturing the setup records the screen-linear uv
(`meshlet::screen_weights`); under the material style's surface or affine
texturing each record carries the local position's and the lit uv's
gradients too. The resolve's fragment stage adds one storage buffer and one
uniform and no texture (the lit stage already samples 15 of 16).
Reflections, views, probe faces, shadows and orthographic cameras keep the
hardware path.

Foliage sways on the meshlet path. A primitive's glTF `_OG_PIVOT` and
`_OG_ANCHOR` (branch pivot and stiffness, leaf anchor and flutter, VEC4
floats) and `extras.ogFoliage.leaves` cook into the model's optional
`foliage_rows` and `foliage` sections: welding keeps vertices of other
pivots apart, a simplified level locks every vertex on a seam between two
pivots or anchors, and leaf cards get no simplified levels.
`RenderScene::set_mesh_foliage` puts a mesh's foliage
(`meshlet::MeshFoliage`) after the vertex pool; every pull moves the vertex
by `foliage_offset` at `Instance::sway`. The cull widens spheres by the
deformation bound (`FoliageBounds::deform_margin`) and takes no cone.

The CPU statement of every test is `meshlet.rs`, whose unit tests check it.

**Runtime geometry and debug draw.** Games make geometry from attribute
arrays (`og.render.bufferGeometry`, three.js BufferGeometry's attributes,
normals computed when missing) and from three.js's primitives: cylinder,
cone, capsule, torus, ring, circle, and a tube along a centripetal
Catmull-Rom curve or an og.path. A dynamic geometry is rewritten in place
each tick (`updateGeometry`). A made geometry is also a physics mesh under
its handle (an og.Collider's `mesh`). Each geometry has a BVH, built when
made or updated; a model's is cooked into its blob (`model { raycast false
}` leaves it out, and the first raycast that reaches the model builds it);
`og.render.raycast` tests what is drawn rather than the colliders. Inside a
system the raycast reads the meshes as the probe recorded them at the
tick's start.

Procedural modelling (`crates/og-geom`) makes three.js's modelling
geometries: shape (earcut, a port of three.js's), extrude (bevel, steps,
along an og.path with Frenet frames), lathe, polyhedron with the four
regular solids, a section swept along a path with scale and twist keyed
over its length, merge of placed parts (mirrored, arrayed), and CSG (union,
subtract, intersect). The crate depends on glam, `robust` (Shewchuk's
predicates) and `spade` (constrained Delaunay of each cut face), never on
og-render. Its mesh converts into a MeshData in og-runtime's `geometry.rs`
and goes the way of `bufferGeometry`. Counts, vertex orders and uvs are
three.js's. Every result is the same on every host: CSG decides each plane
side and edge crossing with exact orient3d and orient2d, pieces are
classified by a winding number, coincident faces keep one copy or none, and
the generators take sine and cosine from series on integer segment indices,
never from libm. `og.Shape` and `og.Path2` are TypeScript (three.js's Shape
and Path); they flatten to points before they cross the ABI.

A model can be written as text from the same parts: an `.ogt` of `type
model` (og-cook's `cook/model_text.rs`) names each part's geometry by its
og.render call and desc fields, places it as mergeGeometries does, gives it
a material and hangs it from a named pivot. The cook writes it as a glTF
binary before the model path reads it (an FBX or OBJ goes the same way,
read by ufbx in `cook/fbx.rs`): meshopt, the LOD chain with measured
errors, the raycast BVH, the node graph. A part placed several times is one
mesh under several nodes, drawn instanced; a mirrored placement is baked.
The cook refuses it past the project's triangle and vertex budgets.
og-geom's `path2.rs` flattens the text's og.Shape calls with the
TypeScript runtime's formulas, bit for bit.

Lines and points are one pass (`lines.rs`, `points.rs`,
`shaders/lines.wgsl`): each segment or point is an instance of a
screen-space quad with a feathered edge, so pixel or world widths and
dashes behave the same on every backend. Material `wireframe` draws its
instances' unique triangle edges through the same pass, computed once per
mesh revision. The pass draws in two places:

- **Scene**: in the main pass after the transparent meshes and particles,
  depth tested or not.
- **Overlay**: over the finished frame, after every post effect and before
  the UI. Debug draw (`og.debug`), the debug views and screen lines draw
  here.

The debug views are a `RenderOptions.debugView` enum (none, physics, nav,
bounds, lod, and the lit image's views); `og.debug.setView` and the command
channel's `debug:` action switch them. Physics is rapier's debug render of
the last step; nav draws grids, navmesh polygons and agents' paths; bounds
draws mesh bounds. The lit image's views replace the lit colour with one
term of it, written by the lit shaders (`RenderScene::debug`, the frame
uniform's `light_color.w`): `cascades`, `ao`, `albedo`, `iblDiffuse`,
`iblSpecular` and `direct`; `aoDepth` and `aoRaw` show the AO pass's input
depth or its term before the blur. Debug draw is not world state, never
feeds the tick, and the `debug` module is left out of publish builds.

Shading diagnostics never silently substitute the beauty image. An
unsupported 3D surface/view pair draws an eight-pixel magenta/black checker.
`dump_render.debugCoverage` lists the supported paths for the selected view;
all others are unsupported. Terrain and grass expose albedo, cascades and
separate diffuse IBL, specular IBL and direct light; terrain's AO is
unsupported. Scatter exposes albedo and screen AO; impostors and custom
shaders expose albedo. Physical materials expose albedo, cascades and screen
AO. Water exposes screen AO; sky, particles, ribbons and 3D fluids use the
checker. Meshlet foliage uses the standard material shader. Decals follow
the receiving surface. Terrain-specific modes are unsupported on
non-terrain surfaces. Sprites, 2D fluids, lines, points and UI are named as
unsupported in the dump. Physics, nav, bounds and lod are overlays over
beauty, covering only colliders, navigation, mesh instances and og.Model
respectively.

**Levels of detail, reflection probes and screen-space reflections.**

- **Mesh LOD.** The cook simplifies every model mesh with meshoptimizer into
  levels, `lods 1 0.5 0.25` by default (`model { lods … lodError … }`;
  `lods 1` turns it off). A glTF's own `MSFT_lod` levels are kept. A level
  is a range of the primitive's indices, and the runtime gives each coarse
  level a compact mesh of its own. The cook measures every level's error:
  the largest distance between the level's surface and the full mesh's,
  both ways, in model units, written as the level's `error` row and listed
  by `og cook --report` (`lodErrors`). The renderer draws the coarsest
  level whose error projects to less than one pixel of the render target's
  height at the near side of the model's bounding sphere
  (`og_render::lod`). The previous level is kept within 15 % of a boundary
  (hysteresis). The projected error is scaled by `og.Model.lodBias ×
  RenderSettings.lodBias` (2 keeps detail twice as far); a bias of 0 draws
  the full mesh. `dump_entity` reports the level drawn (`render.lod`), and
  `debugView lod` tints models by level: green, yellow, orange, red.
- **Reflection probes.** An `og.ReflectionProbe` is a box of the world,
  centred on its entity, whose lit surfaces reflect a cube captured at its
  centre instead of the environment map (`probes.rs`). Each capture is six
  90° views drawn with the lit pipelines in a `reflection-probes` graph
  pass before the main pass: opaque meshes, skinned meshes in their current
  pose, terrain, scattered copies within the box's half extent, the sky,
  water and 3D fluids (a face with either draws its own depth prepass and
  fluid targets and copies itself after the sky, so water and fluids
  refract the face drawn so far; no planar reflection; og-render README,
  Reflection probe faces), then transparent meshes sorted from the probe,
  particles (their own `ParticleViewGpu`), scene lines and points, world
  sprites and text, on every layer; lit by the sun and every local light,
  its own reflections the environment's (one bounce). It is then copied and
  GGX-prefiltered like the environment (`ibl.rs`). The lit shaders
  box-project the reflected ray onto the probe's box and blend into the
  environment over `blendDistance` at its faces. Probes cost no texture
  slot: a draw inside a probe's box binds that probe's frame group, whose
  uniform carries the box and whose two IBL cubes are the environment's
  and the probe's. The probe is chosen per instance (`probe_select.rs`):
  the smallest box holding the centre of its bounds; the instance sort keys
  on it. Skinned instances choose by their posed bounds (each joint's
  bind-pose box through the frame's joint matrices, `skin.rs`); terrain
  levels, scatter chunks, water and 3D fluids per draw. Every lit shader
  samples through `frame.wgsl`'s `ambient_light`/`reflected_light`. At most
  `MAX_REFLECTION_PROBES` (8) probes are used, nearest the camera first.
  - A `baked` probe is captured once. `og bake probes [project]` renders
    them on the native host and writes each into the cook cache as an
    environment blob (`cooked/probes/<key>.ogb`, kind `probe`). The key
    hashes the probe's position, box and face size and its room's content
    by value (`probe_content.rs`), so a changed room is a new key. The
    runtime loads the blob when its key is in the bundle; otherwise the
    probe draws its capture on its first frame. Beside each blob a record
    (`cooked/probes/<key>.ogt`) names the room's model assets with their
    cooked hashes and, when game code built part of it, the hash of `src/`;
    `og doctor --project` reports a changed one as `stale-probe`.
  - A `realtime` probe is drawn again every `interval` ticks.
- **SSR.** `PostEffects.ssr` (`ssrMaxDistance`, `ssrThickness`, `ssrSteps`,
  `ssrFade`, `ssrIntensity`, `ssrMaxRoughness`, `ssrResolution`) is
  stochastic screen-space reflection after AMD FidelityFX SSSR and its
  reflection denoiser (MIT, `third_party/fidelityfx-sssr`) and Frostbite's
  SSR (Stachowiak 2015) (`ssr.rs`, `ssr.wgsl`). One graph pass, `ssr`,
  between the prepass (and AO) and the main pass, records compute passes:
  the min-max depth pyramid of the prepass's view depth; one ray a pixel
  (or a 2x2 block at half resolution, the default, a different pixel each
  frame, with a joint bilateral upsample by depth, normal and roughness and
  a second à-trous prefilter pass on rough lobes) drawn from the GGX
  distribution of visible normals and marched through the pyramid in (uv,
  1/z) with Uludag 2014's min-max Hi-Z, with FidelityFX's hit validation;
  the resolve, weighing neighbours' rays by the pixel's lobe over each
  ray's pdf; FidelityFX's edge-stopping prefilter; and the temporal pass
  (the denoiser's reprojection, history clipped to the neighbourhood's box
  with variance averaged over four frames as SVGF, a still history centred
  on the four-frame mean as NRD's fast history; up to 32 samples on rough
  surfaces, 64 when still). A hit reads the previous frame's lit image
  (TAA's history, or the main pass's copy that screen-space GI reads), so
  a frame after a cut has none. The prepass's depth target is RG32F, its
  second channel the gloss and metalness the lit pass shades. The result
  is a read-only storage texture in the frame group (binding 13, no texture
  slot), blended by `frame.wgsl`'s `ssr_reflected` over the radiance the
  probes, probe GI and environment give, before the split sum, in every
  lit shader that writes the prepass's reflectivity; diffuse light is
  untouched. `Frame.ssr` is on in the main camera's uniform, off in views,
  probe faces and planar reflections. Opaque surfaces of a perspective 3D
  camera only (`RenderCapabilities.ssr` is false only without a renderer).
- **Screen-space GI.** The lazy resource `og.GlobalIllumination`
  (`enabled`, `intensity`, `maxDistance`, `thickness`, `steps`, `rays`,
  `occlusion`, `resolution`, and `allowed`, the quality switch
  `og.render.applyQuality` writes), registered on first use, draws in the
  AO passes' place (`ssgi.rs`, `ssgi.wgsl`): the AO targets become
  RGBA16F, r the occlusion, gba the bounce, read through the effect
  group's AO texture (no lit binding added; the lit stages sit at 16 of 16
  sampled textures). Each pixel casts cosine-weighted rays from its prepass
  normal and marches them across the screen (perspective-correct, each cut
  where it leaves the frustum, `maxDistance` 16 m capping it). A hit on a
  surface facing the ray reads the previous frame's lit image where the
  hit pixel's velocity puts it (TAA's history under TAA, else a kept copy
  of the main pass's image): one bounce a frame, many over a few. Under
  TAA the noise moves every frame and TAA is the denoiser; the blur and
  half-resolution upsample are the AO's, over four channels, weighted by
  normals. The bounce adds to the diffuse ambient light times the albedo;
  the hit rays take their share of the ambient light (or the AO term does
  with `PostEffects.ao`). Scattered copies keep the AO alone.
- **Probe GI and sky occlusion.** `og.GlobalIllumination.method` `probes`
  (or `both`, with the screen rays over it) lights every lit surface's
  ambient light from DDGI probes (Wicked Engine's, MIT,
  `third_party/wicked-ddgi`; `gi_probes.rs`, `gi_*.wgsl`).
  - **Probes.** Cascades of 16 × 8 × 16 probes scroll with the camera,
    `probeSpacing` apart in the first cascade and twice as far in each
    next, each centred 0.3125 of its extent ahead of the camera. Cascade c
    relights 1/2^c of its probes a frame.
  - **Tracing.** Each probe traces `probeRays` rays in software through a
    two-level BVH of the scene, simplified to at most 8192 triangles a
    mesh and 129² samples a terrain (`gi_scene.rs`); WebGPU has no hardware
    ray tracing. A hit is shaded with the sun, one shadowed local light,
    the hemisphere light, emission and the probes' previous bounce.
  - **Probe data.** Each probe keeps deringed L1 harmonics and a 12²
    octahedral map of distance moments. Lit fragments read 8 probes a
    cascade, weighed by a Chebyshev visibility test so light does not leak
    through walls. A probe whose rays find it inside geometry is left out
    and moved out through the nearest back face; its next update starts
    its light and distances again.
  - **Sampling (`gi_sample.wgsl`).** A lit fragment gathers the 8 probes
    around it in each cascade once, into their weighted harmonics,
    evaluated along its normal for diffuse light and along the reflected
    ray for the reflections' occlusion; the ring buffer's slot and key are
    worked out for the base corner alone.
  - **Bindings.** The probes share the light volumes' storage buffer
    (frame binding 10); nothing beyond the default limits is requested.
    The effect passes bind it too (binding 7), for the screen GI.
  - **With the screen GI (`both`).** The screen rays reach one probe
    spacing at most, and their hits replace the probes' light from the
    directions they hit: the screen GI's pass takes the probes' L1 light
    along the hits (h c0 + 1.5 c1·s over π, with h the hits' share and s
    their mean direction).
  - **Sky occlusion.** The sky reaches a point only through the probes'
    rays that miss. Reflections outside every reflection probe are scaled
    by what the probes see along the reflected ray (`gi_reflected_from`).
  - **GTAO bent normals (XeGTAO, MIT, `third_party/xegtao`).** With
    `PostEffects.ao`'s GTAO on, the AO targets become RGBA16F and carry the
    world bent normal in gba; the lit passes read the probes along it. The
    screen GI's bounce and the contact shadows' masks use the same
    channels, so either turns the bent normals off.
- **The live sky.** While the background is the per-pixel sky
  (`Background::Procedural`: og.Sky, og.DayNight, og.PhysicalSky,
  og.Clouds) and no environment map replaces it, the image-based light and
  the reflections' fallback follow the sky as drawn: Unreal's real-time sky
  light capture, ported (`sky_capture.rs`). `fs_sky_look` draws the sky
  into a 128² cube (stars off), the reflection probes' filter makes the mip
  chain, irradiance and GGX levels in cubes of its own, and a copy puts
  them into the cubes the frame binds once whole: no binding added, never
  a half-made sky. The work is spread over eight frames
  (`FramePass::LiveSky`, timed as `live-sky`); the first frame of a live
  sky does all eight, and a change reaches the light within sixteen
  frames. The capture's clouds take the last capture's irradiance as
  ambient light.
- **Physical solar radiance and cloud view fog.** The physical sky adds the
  atmosphere-attenuated solar radiance after scattering, with limb
  darkening and the disc boundary filtered over one pixel's angular
  footprint; its aureole comes from scattering and bloom. Only the
  scattered sky keeps the look's saturated-colour shoulder; the sun reaches
  bloom and the tonemapper in scene-linear HDR
  (`third_party/hillaire-sky.md`). The sky output is bounded to finite
  RGBA16F; the HDR output mapper bounds it to the display peak. Cloud
  scattering samples take the same distance fog as meshes, and the height
  fog's column by `Fog.sky`'s share as the sky does.
- **Aerial perspective and far clouds.** Under the physical sky the air
  between the eye and a cloud is integrated with the sky's own scattering
  (Hillaire 2020 §5.5, `sky_scatter.wgsl` `air_path`, at the cloud's
  transmittance-weighted depth), the sun shaded by the cover
  (`Clouds::light`) and, under a closed deck, the sky's light by the deck's
  base. Past the march's reach (30 km of marching from the layer's entry,
  60 km to the entry at most) the layer is drawn as its mean: a ray meets
  cloud with probability `1 - (1 - cover)^(path / cell)`, lit as the
  march. The gradient sky keeps its 27 km fade.
- **Lens flare through clouds.** With clouds over the procedural sky the
  flare's occlusion pass reads the disc as the sky drew it against its
  expected luminance (`lens_flare.wgsl` `fs_flare_occlusion`), so a cloud
  over the sun dims the flare by its transmittance.
- **Realtime light volumes.** The lazy component `og.RealtimeLightVolume`
  (`updatesPerFrame`, `intensity`) beside an `og.LightVolume` relights that
  box's probes at run time (`probe_grid.rs`, `probe_grid.wgsl`): a 16² cube
  capture through the reflection probes' capture path, projected onto L1
  harmonics by a compute pass straight into the light volumes' storage
  buffer (frame binding 10), after the baked volumes; its header entry
  carries bit 21 of the packed counts. Up to eight probes a frame over
  every volume, in a stride order; the captures sample the probes, so each
  sweep adds a bounce. A volume is sampled once swept; until then, and
  while the quality's GI switch is off, its box takes the baked probes or
  the environment. `ambient_irradiance` blends realtime over baked over the
  environment by the same edge weight; lightmapped surfaces keep their
  lightmaps. Realtime probes relight reflections (`volume_reflected`): the
  probes' L1 irradiance along the reflected ray; inside a reflection
  probe's box smooth surfaces keep its capture, blurring into the probes'
  as roughness rises. The projection derings each channel (|c1| ≤ c0).
  Baked volumes leave reflections as they were. A realtime volume is whole
  up to its faces and fades out over a spacing beyond them (a baked one
  fades in over its outer spacing). The low preset's GI switch waits in
  the presentation state (`FxState::gi_preset`) until a world names
  `og.GlobalIllumination`.

**Render targets, views, layers and custom shaders.**

**Render targets.** A render target (`targets.rs`) is two entries in the
scene's texture table: its colour and, optionally, its depth. Any material
slot, sprite or UI image samples it like any other texture. Snapshot and
restore keep the targets; a restored renderer draws every view on its next
frame.
- `ldr` (Rgba8 sRGB) holds the view's finished image.
- `hdr` (Rgba16Float) holds its linear radiance; the frame that shows it
  exposes it.
- The depth texture: the view draws its opaque meshes, skinned meshes and
  terrain again through the prepass pipelines into a single-sampled
  linear-depth colour target (RG32F, as the main prepass's `lindepth`),
  normalised from near to far. `RenderCapabilities.renderTargetDepth` is
  true with a renderer.

**Views.** A view (`views.rs`, `og.CameraView`, `og.Mirror`) is a secondary
camera, a timed graph pass `view-<i>` drawn before the main pass in
`order`, so a target shows this frame's image. Each has its own frame and
effect uniforms, light clusters from the frame's lights, HDR and depth
attachments kept while it exists, and a resolve into its destination (a
target: exposed and tone mapped for ldr, linear for hdr; a viewport rect:
after the post chain, under lines, the overlay and the UI).

A view draws what the main pass draws, from its eye and on its layers:
opaque and transparent instances (sorted from its eye), skinned instances,
terrain, sky, particles (its own `ParticleViewGpu`), scene lines and points
(its own `LineUniform`), world sprites and text, and trails, ribbons and
beams (`Trails::emit` facing its eye). Its lit sprite layers multiply with
its own 2D light map (`Light2dRenderer::write_view`, its size times
`Lighting2d.resolution`), and its 2D fluids are composited from field
targets of its own size. With soft particles every view runs its depth
prepass. With `PostEffects.glow` its glowing sprites draw into a half-size
glow chain through the frame's bloom filters (`bloom_chain.wgsl`), added
before the resolve at the frame's glow weight (`views_glow.wgsl`). Screen
sprites, the UI and the post chain are the main frame's; an `ldr` target or
viewport carries exposure and the tone curve, an `hdr` target linear
radiance. A view draws everything its mask admits, so `layers` and `every`
are the knobs. Under GPU culling (`cull_gpu.rs`) each view is a pass of its
own: its planes and layer mask, its opaque instances drawn indirect;
transparent and skinned instances keep CPU lists. Skinned instances and
baked crowds are culled by their posed boxes on both paths (joints'
bind-pose boxes moved by the palette; a crowd's clip's over all frames).

Crowds (`RenderScene::crowds`, og-render's `crowd.rs`, what
og.CrowdAnimation members draw as) are the GPU-driven skinned workload: per
culled pass a compute pass (`crowd_cull.wgsl`) tests each member's clip box
against the pass's planes (the camera's also against the Hi-Z pyramid),
picks its level (lod.rs's rule), and compacts survivors by a deterministic
prefix sum into indexed indirect draws that blend the clip's baked frames
in the vertex stage. A crowd's last level may be an impostor
(`CrowdImpostor`): cards of its full mesh baked on the GPU per view around
the vertical and per clip frame (`crowd_impostor.rs`), coloured per part by
each crowd's materials, lit, depth-written and cast into shadows. Without
`INDIRECT_FIRST_INSTANCE` (`RenderInfo::gpu_crowds`, capability
`gpuCrowds`), or with secondary views or probe faces, crowds take CPU
lists: the same members at the same levels.
- `every` skips ticks.
- A mirror reflects the main eye in its plane through an off-axis frustum
  fitted to the quad's corners (Kooima), the near plane on the glass.
- The id pass (`id_pass.rs`) draws from any camera on its layers
  (`render_ids_from`, `view_id_cameras`). The probe's `describe` lists the
  main camera's entities, each naming the views that also show it, and
  each view with what it shows (§3.4).

**Layers.** Layers are three.js masks: bit n is layer n, everything
defaults to 1, and a camera draws an object when `a & b != 0`.
`RenderScene::instance_layers` and `skinned_layers` split runs and batches
by mask; `sprite_layers`, `line_layers`, `point_layers` and
`particle_layers` carry the other drawables' masks, and every camera tests
them. og-runtime fills them from the `layers` of og.Mesh, og.Model,
og.Sprite, og.Lines, og.Points and og.ParticleSystem. `og.render.raycast`
makes the same test. Shadow maps take every caster.

**Full custom shaders.** `type shader` with `mode full [lit|unlit]`
(`custom.rs`, `shaders/custom_*.wgsl`) has its own `vertex` and `fragment`
entries over a fixed binding contract:
- `camera()`;
- the model matrix, with the skin folded in and morphs applied;
- `Params`;
- `tex0` to `tex7`, the material's eight texture slots;
- the `standard_vertex`, `standard_surface` and `noise3` helpers.

The modules keep the built-in entry names and vertex buffers, so they draw
through the hooked materials' pipelines. The prepass, every shadow pass,
the water's reflection and every view run the user's vertex entry and cut
by its alpha. The cook validates the modules (WGSL, SPIR-V, MSL, GLSL ES)
with naga, as for hooks.

**Custom post passes.** A `type post` asset (`post_custom.rs`, an
og.PostPass blob) is a WGSL `post` function reading the previous pass, the
lit image, the prepass's depth and normals, and its parameters.
`PostEffects.custom` is a list of up to eight passes (`List.Post.8`:
`MAX_CUSTOM_POSTS`, `RenderCapabilities.maxCustomPosts`), run in order (a 0
skipped) as `custom-post.<i>` after motion blur and before exposure, bloom
and the tone curve. Each compiles in an error scope; one the device refuses
is skipped and reported (`og.render.postErrors`).

**2D lighting.** `light2d.rs` draws the light map lit sprite layers
multiply with: cleared to the ambient, each `Light2d` adding an instanced
quad over its radius, one strip per receiver light mask side by side in one
target (`LightSlots`, at most four).
- **Shadows.** Each shadow-casting light owns a row of a polar shadow map
  (`light2d_shadow.wgsl`, `Lighting2D.shadowAngles` texels wide, RG32F),
  traced once a frame against the edges of the occluders that reach the
  light (boxes and polygons, concave too). Each texel keeps the first hit
  and the far side of the first occluder, so a pixel inside an occluder is
  not shadowed by it; the cost per light-map pixel does not grow with the
  occluders. The light pass samples the row with PCSS (blocker search over
  the light's angular size, then a filter of the penumbra's width). The
  column wraps at ±π.
- **Normal maps.** A texture with a `<id>#normal` (`normal auto`, made by
  the cook from the silhouette in `normal2d.rs`, or `normal "file"`) is
  linked by the runtime (`RenderScene::set_texture_normal`). While any
  texture has one, the light pass also writes `lightdir` (`fs_dir`): each
  light's luminance-weighted unit direction towards the light at
  `Light2D.height`. Its draws (`sprite_normals.rs`) light by
  `ambient + (L − ambient)·(n·d + ½)/(d.z + ½)`, with `n` the map's normal
  turned into the world by the uv's screen derivatives. A flat normal gives
  exactly `L`. A frame without normal maps makes no `lightdir`.
- **`og.render.light2dAt`** (og-runtime `light2d_at.rs`) answers the light
  at a point for game rules: the same falloff, cone and fade, hard shadows
  through the same outlines, in og-core's deterministic math, the same bits
  on every host.

**Baked lighting, volumetrics and area lights.**

- **Baked lighting** (`og bake lightmaps [project]`, the og-bake crate). A
  native CPU tool: a progressive path tracer over the scene's models and
  its directional, point, spot and rect lights, with a BVH and a fixed
  seed, so a bake with a given `--samples` is repeatable. Entities marked
  `static true` get lightmap UVs from xatlas at one texel density
  (`--texels-per-metre`). Each texel stores the indirect light arriving
  there (bounces, the sky, emissive surfaces); a light's direct term is
  never baked. Each `og.LightVolume` box gets a grid of L1
  spherical-harmonic probes. The bake writes `<scene>.lightmap.ogt` (the
  remapped parts, the probes, the scene and model hashes) and
  `<scene>.lightmap.ktx2` (RGBA16F array layers) beside the scene; the
  cook turns them into an `og.Lightmap` blob. When a scene is instantiated
  the runtime loads its bake if the hashes match; static models then draw
  lightmapped copies of their parts (a second vertex stream, their scene's
  pages at group 3; `lightmap.rs`, `lightmap.wgsl`). Each scene's pages are
  one texture array with its own bind group, so scenes loaded together all
  keep their bakes. A bake stops drawing while its static models are all
  despawned. Other lit fragments inside a volume take their ambient light
  from the probes, trilinear and faded over one spacing (frame binding 10,
  `ambient_irradiance`). Without a bake, or with a stale one (the scene's
  source text or a static model's geometry changed), the scene renders
  unbaked; `og.render.bakedLighting` gives the reason and `og doctor
  --project` warns `unbaked`. `RenderSettings.bakedLighting` false draws
  every scene unbaked. A static mesh draws its bake in every lit batch
  (standard, lean, compact, texture-array, material-shader, physical).
- **Projection painting** (`og bake projection <step> --run <dir>`,
  `og-bake/src/projection`). A native CPU tool that turns a gray level glb
  into a textured one by painting it from views: the trompe workflow
  (Patch-gg/trompe, Apache-2.0) without Blender or a GPU. The image model
  stays outside the engine; the agent runs the steps and
  calls it between `prepare` and `project`. `init` rasterises the level's
  TEXCOORD_0 into texels (world position, normal, triangle) with a BVH (a
  level without TEXCOORD_0 is charted by xatlas into one page first);
  `plan` picks cameras in paint order from walkable points on the floors
  (ground, then roof passes and an aerial ring) by greedy coverage of a
  probe render; `prepare` ray-casts a view, renders its clay (ambient
  occlusion and a key light) and composites the paint so far into
  `state.png` with `edit.png` (white where the view sees no paint) and a
  mode (`first`, `fill`, `roof`) for the prompt; `project` takes the
  model's picture, keeps the known pixels, colour-corrects and feathers the
  new ones into them, drops what the model left unpainted, and writes each
  visible texel whose view is better than the one that painted it (visible
  when the pixel's ray hits its triangle, or within 1 cm and 0.5 % of depth
  with agreeing normals; silhouette pixels count less); `holes` adds the
  view that sees the most unpainted texels; `export` pads the uv gutters
  and writes the level with the texture as every material's base colour.
  The run directory holds everything (`run.json`, `plan.json`,
  `texels.bin`, `paint.bin`, `views/K/`), so a run resumes and a view can
  be re-projected.
- **Volumetric fog** (`og.Fog.volumetric`, `volumetric.rs`). The fog's
  density, height fog, `scattering` and `anisotropy` (Henyey–Greenstein)
  define a participating medium up to `volumetricDistance`. By default it
  is a froxel grid (160 × 90 × 64, slices deepening quadratically) in a
  named `volumetric` compute pass, each cell lit by the sun through its
  cascades and by the cluster's point, spot and rect lights through their
  shadows and cookies (the mean of eight stratified, jittered samples over
  its slice's depth), then integrated front to back. The `raymarch`
  method marches each pixel's ray in 24 steps, each lit by the sun and the
  first 8 lights of its cluster (`RAYMARCH_LIGHTS`).
  `RenderCapabilities.volumetricLights` reports the per-sample bound (64
  froxel, 8 raymarch); `RenderDesc::volumetric`
  (`OG_GFX_VOLUMETRIC=raymarch`) selects the raymarch, and
  `RenderCapabilities.volumetricFog` reports the method. Both blend over
  the lit image after the main pass as colour × transmittance plus
  in-scattered light, reading the prepass's linear depth.
- **Area lights** (`og.RectLight`: width, height, colour, intensity,
  `twoSided`; `area.rs`). Clustered with point and spot lights and shaded
  with linearly transformed cosines (Heitz et al.'s GGX fit,
  `third_party/ltc`, 32 × 32 half floats in one 16 KiB uniform). With
  `castShadow` one takes a layer of the local shadow array (two when
  two-sided, counted in the shadowed lights' budget): a 150° perspective
  view from its centre along its normal (`RECT_SHADOW_FOV`), read as PCSS
  (`rect_shadow`: 16 taps for the mean blocker distance, then 24 bilinear
  taps on a per-pixel turned spiral over the penumbra half-size ×
  (receiver − blocker) / blocker). Receivers past 75° from the normal stay
  unshadowed. Terrain, scattered copies, 3D fluids and full custom lit
  shaders shade them with LTC and their shadow (and point and spot
  cookies) through `area.wgsl`'s `surface_local_light`.
- **Light cookies** (`SpotLight.cookie`, `PointLight.cookie`, a cube of six
  faces stacked in one texture). A small pass draws each cookie into layers
  of the local shadow array past the shadow layers, each texel packed as
  8:8:8 in the R32Float array, so a cookie takes no texture slot. At most
  24 layers (`MAX_COOKIE_LAYERS`, `RenderCapabilities.maxLightCookies`):
  one per spot cookie, six per point cookie, a texture shared by lights of
  one kind once. Cookies past them, in their lights' entity order, are not
  drawn; the frame warns once, naming the lights (`og_render::cookie_layers`).

**three.js parity for ports.** A game moved from three.js keeps its look
through opt-in fields that reproduce what three.js and its common passes
draw, each off by default. Lighting: `Environment.hemiSky/hemiGround/
hemiIntensity` is HemisphereLight (diffuse-only, darkened by AO);
`DirectionalLight.shadowBox` replaces the cascades with one fixed box around
a focus ahead of and below the camera (`shadowFocus`, `shadowFocusDrop`),
with three.js' `bias` and `normalBias` (`shadowBias`, `shadowNormalBias`).
Post: `PostEffects.aoMethod screen` is depth-only screen-space AO,
`aoRadiusScale` grows the AO radius with distance, `aoDirect` lets AO darken
direct and emitted light (above 0 also water surfaces), `sharpen` is a CAS
pass after the output, `RenderSettings.bloomRadius/bloomContrast` switch
the bloom to UnrealBloomPass. Water: `Water.geometry` draws over any mesh
instead of the grid; `f0`, `reflectionBias` and `glint` are Water.js's
Fresnel floor and sun glint; `normalMap/normalScale/flow` are Water.js's and
Water2's normal-map ripples. Materials: `alphaToCoverage` (with MSAA),
scatter included; material shaders get `Hook.eye`, `Hook.front`,
`planar_reflection(p, shift)` (with `RenderOptions.planarReflections 2`) and
`screen_ao(p)`. `Terrain.visible false` keeps a terrain as ground for its
scatter only; `TerrainScatter.castShadows` turns the copies' shadows off.
og-ui's style `ramp` takes a fifth number. `color_textures uastc` (§2.5).

**Render origin.** The frame is drawn relative to a point near the camera,
so the GPU's f32 arithmetic works on numbers within a few kilometres
(`og-render/src/origin.rs`). Extraction builds the frame in the world, then
moves it to the render origin, the point of an 8192 m grid nearest the
camera (`extract::render_origin`): a power-of-two grid, so `p - origin` is
exact near it, and a camera within 4096 m of the world's zero draws at it.
`RenderScene::rebase` moves every world-space position the frame draws
(instances and their previous transforms, cameras and views, lights,
sprites, text, particle systems and colliders, trails, ribbons, beams, scene
lines and points, decals, probes and light volumes, water, terrains, fog
heights, 2D lights, fluids) and records the origin in
`RenderScene::origin`; screen-space sets and the overlay stay. The camera's
look comes from its rotation, not from a target point. What the renderer
keeps between frames moves with the origin, so a move is invisible: TAA's
history and the previous view-projection (`origin::shift_matrix`),
world-space particles and trail histories. Patterns laid in the world add
the origin back, in f64 on the CPU and without whole repeats: terrain layer
uvs, the lake's normal-map layers, caustic cells, particle turbulence, the
stylised surface origin, the shadow cascades' texel grid, and the wind
phase of scattered copies. Material hooks read `h.origin` and full custom
shaders `camera().origin`. The origin never reaches the tick; keeping the
simulation precise is `og.WorldOrigin` (§2.4).

**Shared frames.** A native engine process can show its frames in another
process's window with no CPU copy: Origin Desktop's Preview, while its
runner runs a Session's engine. The runner names the window's socket in
`ORIGIN_FRAMES_SOCKET`; with it set, the native host (`og run`, `og dev`)
draws in the slots' format and links its renderer (og-render's
`shared_frames`, `FrameLink`) to the socket: five shareable textures per
generation (IOSurfaces by Mach rendezvous on macOS, Vulkan opaque fds on
Linux, D3D12 NT handles on Windows) sent with one `Hello`, each loop frame
copied on the GPU into a free slot and announced `Ready` when the copy has
finished, `Release` giving slots back, `Resize` applied before the next
frame and answered with `Rehello` and the next generation. The wire is the
desktop's `shared-frame` contract (origin-agent
`desktop/crates/shared-frame`), implemented here; neither side links the
other. The engine never waits for the window: the handshake runs on the
link's thread, a frame with no free slot is not shown there, and a window
that is not listening leaves the process drawing offscreen, reconnecting
when it comes back. Synchronisation is the CPU's (the copy's completion,
the window's release after its own later frame).

The socket also carries an opt-in accessibility extension. Messages are a
`u32` little-endian byte length followed by serde JSON. After `Hello` the
window sends `{"AccessibilitySubscribe":{"version":1}}`. Version 1 pins the
serde schemas of AccessKit 0.25.2 (`TreeUpdate`, `ActionRequest`):

- Engine → window: `{"AccessibilityUpdate":{"version":1,"generation":0,"seq":42,"update":<TreeUpdate>}}`.
- Window → engine: `{"AccessibilityAction":{"version":1,"request":<ActionRequest>}}`.

No subscription means no accessibility traffic; unsupported versions are
ignored. A connection starts with a full tree, then only changed nodes and
focus. Node ids follow UI paths and are never reassigned within a
connection. Removed children disappear from their parent's child list.
Reconnection resets the tree and requires a new subscription. Bounds use
the shared frame's physical pixels, before the window's placement
transform. `generation` and `seq` identify the frame; updates and `Ready`
may arrive in either order. Apply every incremental update in socket order,
even if the window skips the image.

The engine accepts focus, click, set-value, increment, decrement,
scroll-into-view and directional page-scroll actions, queued between frames
and applied in the next UI tick after checking the focus scope, disabled
state and target existence. Numeric controls keep their bounds and step;
text edits keep their length limit and undo history. The engine owns no OS
window: Origin Desktop owns the AccessKit platform adapter (platform ask
25). The web shell's DOM mirror is separate.

### 2.7 Animation, physics, navigation, audio, UI

- **Animation**: glTF skins and clips, a blend tree asset (`.ogt`), state
  machines as data, root motion, additive layers, two-bone and FABRIK IK,
  blendshapes. The cook diagnoses each source mesh/skin binding before
  welding or LODs, on incremental cooks too. `skinDiagnostics` in `og cook
  --report` and the asset notes count vertices with summed weight >0.02 on
  another limb than the dominant one, excluding limbs with the same attach
  joint (information, not a warning). Humanoid regions and probes use the
  retarget map (including `animation { rig }`): each upper arm turns −70°
  about model X, its forearm −40°, the opposite arm +10°; stride turns left
  upper/lower leg −35°/+30° and right upper leg +25°. Rotations compose
  down the hierarchy about inverse-bind joint positions, +Y up and +Z
  forward. Without all four mapped limb roots, the root's longest joint
  chain is the spine; side branches of at least three joints are regions,
  and each such branch and the continuing spine child at its fork gets a
  −70° probe named by its skin joint index. Only triangles whose posed
  longest edge is both >2.5× its bind longest edge and >0.08 model units
  warn, naming the skin, mesh, primitive, probe and zero-based source
  triangle ids. Nothing is changed; these are fixed stress probes, not a
  proof that every animation is free of stretching.
- **Destruction** (`og.destruction`): props are prefractured in the cook,
  never at run time. A `fracture { }` block cuts a model into Voronoi cells
  clipped by exact CSG (og-geom `fracture`, cut faces tagged with their
  neighbour and given the interior material), with optional children per
  chunk; `<model>#fracture` holds the chunks' hulls, hierarchy, the support
  graph's bonds and the anchor faces (og-destruct `FractureAsset`). In the
  tick, before og.physics, damage wears bond health (Blast's model of
  health as effective area), the support graph splits into islands, and
  Blast's stress solver (CGNR over the anchored islands, ported to f32 with
  fixed orders, the world a static node mirrored through each anchor face)
  damages what is past its elastic limits. An island whose membership
  changed is rebuilt as one body: static colliders while anchored, else a
  dynamic og.RigidBody on its first chunk with the rest as compound
  children. Bond health, chunk entities and queued calls are a state block.
  Fine debris is visual only: a GPU mesh-particle burst replaces pieces
  below og.Destruction.debrisVolume, which never become bodies.
- **Motion generation** (`og.motion`, og-motion): a humanoid is animated by
  intent instead of hand-authored clips: game code queues goals (a pose,
  a place and heading, a turn, a walk or run to a point) and the engine's
  motion model, a student distilled from NVIDIA MotionBricks, fills in the
  motion between the character's recent poses and those goals, a short span
  at a time. The model runs on the device in Rust on wgpu compute (one
  implementation for both hosts, no third-party inference runtime, no CPU
  path). Its poses are presentation (§2.4): the simulation moves the
  character by its controller deterministically, the pose follows, and
  gameplay never reads it; og-anim retargets onto the game's rig and plants
  the feet from the model's contact output. Its weights ship inside the
  engine; a game that uses `og.motion` cooks them as an optional pack,
  named in the manifest outside its outputs and never in the boot bundle,
  read and uploaded in the background once the bundle loads.
  `motion.presentation(world, entity)` reports `loading`, `warming`,
  `ready` or `failed` with a reason on both hosts; ready means generated
  frames and their current palette, never an authored fallback. It depends
  on GPU timing: game logic must not branch on it, and no component, goal,
  tick, snapshot or hash depends on it. The page or shell cover may wait
  for ready or failed; there is no per-entity reveal path. `motion.status`
  is the deterministic goals; `capabilities.backend` means weight
  residency, not first-pose readiness. The probe's `dump_motion` reports
  each agent's `presentation`, also before the model is resident, and
  `model.startup` records wall milliseconds: native pipeline creation
  (`compileMs`), weight/mask preparation and upload submission
  (`uploadSubmitMs`, not a completion fence), residency to first batch
  scheduling (`firstDispatchDelayMs`), first batch scheduling through
  readback (`firstInferenceWallMs`), time polling that batch
  (`firstInferencePollMs`), and readback to the first generated palette
  (`firstPaletteDelayMs`, including decode); compile and upload are null on
  the web, unfinished phases null. Inference is sliced at 3 ms.
  - **Jumps: the simulation owns the elevation.** `motion.jump` is a goal
    like the others. After its `prepare` seconds the og.motion system takes
    off in the tick: with an og.CharacterController through the
    controller's own jump (its `jumpSpeed` set for the goal's `height`
    under the controller's gravity, held on the takeoff tick), so ceilings
    cut the rise and the landing is wherever the controller reaches ground
    (a ledge, a pit); without one, by the fixed ballistic arc of `height`
    back to the start height. The goal ends `recover` seconds after the
    landing tick. Takeoff, height, flight time and landing are never read
    from the model, and the jump's state is in snapshots and the hash. The
    model's own vertical arc is not trusted for this: with the tokens the
    student chooses, generated hips sit up to 74–79 cm off the source's in
    flight while the source-encoded tokens stay within 11 cm
    (og-motion-train `data/trajectory-pilot6/GEOMETRY.md`); its conditions
    do not determine the arc.
  - **The generated jump is drawn in place, on the simulation's flight.**
    The drawn hips are the Transform's height plus the pose's local
    height: the generated hips' height minus how far its lowest foot is
    above that foot's ground clearance. It equals the generated height
    whenever a foot is down, so the crouch before takeoff and the
    compression after landing stay the model's; in flight the lowest foot
    rides the Transform, so the feet never pass below what the controller
    cleared and reach the ground on the landing tick. No generated vertical
    travel is added to the controller's. The clip is retimed onto the
    simulation's (distance matching): its frames before its own takeoff
    (its contact output: the first run of no contact at least 0.4 of the
    simulation's flight long, not a stride's) spread over `prepare`; in
    the air its flight frames follow the share of the flight the tick
    predicts is done, air time over air time plus time to landing, held
    below touchdown until the landing tick; from the landing tick its
    touchdown frames play. Each tick predicts a controller jump's landing
    by stepping its ballistic arc and tracing the ground under each step
    (the colliders the tick recorded), from where its feet stood at
    takeoff: a ledge up lands early and a drop late with no snap at the
    end, and a ceiling, which cuts the rise, moves the prediction with
    it. Without a controller the landing is the fixed arc's. The drawn
    frame reaches the retimed one at no more than three times the clip's
    rate.
  - **A jump's air pose is bounded toward a landing.** The student
    generates a running jump as a dive: the body pitched toward
    horizontal and both legs swept back up to the hips' height (its
    training data has no jump windows from the original library). Every
    frame of a jump clip leans at most 30° (the pelvis to the chest, then
    the hips to the head); off the ground the bound eases (smoothstep)
    from the takeoff to the clip's own apex, half its flight, to 10°, and
    each leg's swing back from straight below its hip is bounded too, the
    leg further forward from 15° to 0° and the other from 45° to 5°, held
    to touchdown (`og_motion::flight::air_pose`). A leg past its bound turns
    forward at the hip, about the body's side axis, so its knee keeps the
    bend the model gave it. A running jump then leads with one leg, and
    from the apex on the feet are under the hips and the pelvis near
    upright, as in commercial running jumps.
  - **The streaming boundary holds the whole jump.** The presenter asks
    for a jump when its prediction of the mover reaches it (the agent's
    lookahead, which `prepare` adds to) and keeps its generated frames
    from its start through its touchdown and recovery, since the retiming
    needs the clip's own takeoff and touchdown; the frames it holds count
    as generated ahead, so no further goal is asked for while they last.
    When the jump starts with frames already generated past it toward
    another goal (a walk the model took longer over) and no jump asked
    for, the clip keeps one frame past the drawn one (a latency's worth on
    the web) and the model starts again from those, so the jump is
    generated from the jump's start. A flight that outruns the generated
    frames holds the latest generated flight frame and never invents a
    landing. A `pose` goal of `jump` is different: a hop drawn by the
    model alone, the Transform on the ground, never read by gameplay.
  - **Feet are planted on a hinge knee.** The planter's two-bone solve
    bends each knee in the plane its generated knee last bent in (a
    straight generated leg names none) and eases a planted ankle near
    straight-leg reach with soft IK (Nicholas 2009) over the last 2% of
    the leg, so a knee neither turns sideways or backwards nor snaps
    straight in one frame.
- **Physics**: rapier3d (and rapier2d for 2D projects), fixed-tick stepping
  inside the engine tick, a kinematic character controller. A kinematic
  body with an `og.Velocity` moves by it each tick and its Transform is
  written back (one without stays where its Transform puts it).
- **Navigation**: navmesh baked at cook time or at runtime for procedural
  levels; path queries and crowd steering as engine systems.
- **Audio**: the mixer runs in wasm; on the web the engine Worker feeds an
  AudioWorklet through a SharedArrayBuffer ring, natively cpal. Buses,
  spatialisation, QOA streaming, one-shot and looped sources.
- **UI**: designed in `docs/ui.md`; this bullet states the engine side.
  Immediate-mode (Clay-like layout) rendered by the engine; element ids are
  stable strings so the command channel can target them (`touch_move_to`,
  `move_to_text`, `dump_ui`). A game's UI is the game's, never the
  platform's: no platform token reaches it. A game dresses its UI in a
  **style** (`type style`, `og.load.style`), one complete asset: the
  scheme, every palette role, the four radii (small, control, bar, panel),
  the type ramp, fonts, icons, nine-slice frames per element class,
  ornaments, motion and sounds. A look's `ui` line names it: a built-in by
  slug (`templates/styles/<slug>/`, cooked under `@styles/`) or the game's
  own `styles/<name>/style.ogt`. Without one the UI draws in the engine's
  default style, generated from `packages/design` into
  `crates/og-ui/src/theme.rs`. Contrast and accessibility checks run over
  every built-in style (`pack_contrast.rs`). A nine-slice's middle squeezed
  into a control shorter than its art shows the centre of the art's
  middle, squeezed at most twice, and sprites and UI images are sampled
  anisotropically where the device can
  (`RenderCapabilities.spriteAnisotropy`, 16; 1 on a software rasterizer);
  text keeps a trilinear sampler. Screen transitions, count-ups and bursts
  run by the tick in the UI state.
- **Timeline and paths** (`crates/og-timeline`): a path is a curve through
  or controlled by points (`catmullRom`, `bezier`, `linear`, open or
  closed), measured by an arc-length table so a parameter 0..1 moves at an
  even speed; `og.path.create` makes one, a scene declares named ones
  (`path "<name>" { point x y z ... }`), and `og.PathFollower`,
  `og.CameraRail.path` and a timeline's camera ride them. A **timeline** is
  a `type timeline` `.ogt` asset, cooked to an `og.TimelineAsset` blob with
  its caption files (`.srt`, word-timestamp `.json`) imported: a `duration`
  and tracks of keys (camera on keys, a rail or following an entity;
  transform; animation clip; audio; music with fades; caption; event, a
  command-channel action; transition over og.ui's screen cover; marker).
  `og.Timeline` plays one through the `og.timeline` system, after the
  game's systems and tweens and before og.anim and physics. Every value is
  a function of the timeline's time alone, a whole number of thousandths of
  a tick (a tick adds `round(speed × 1000)`, a seek sets `round(t × rate ×
  1000)`), so seeking to t and playing to t give the same world hash: the
  video producer (§2.9) starts at any frame. Playback state is a state
  block; a camera track owns `og.Camera` while it plays and hands back the
  camera it found. Command channel: `timeline:play:<name>`
  (`timelines/<name>.ogt`, on an entity `timeline-<name>`),
  `timeline:seek:<s>`, `pause`, `resume`, `stop`.
- **2D skeletal animation** (og-anim `rig2d.rs`, `skeleton2d.rs`): a `type
  rig2d` `.ogt` asset (pixels, +y up, degrees counter-clockwise, `unit`
  pixels per world unit): bones parent first (rotation, skew, scale), slots
  in draw order (colour multiplier and offset), `region`, weighted `mesh`,
  `box` (a bounding polygon) and `child` (a child `armature` of the same
  rig, posed and drawn inside its slot) attachments, `ik` constraints over a
  chain of any length (one bone points, two are solved exactly, three or
  more by FABRIK seeded on the `bend` side, no trigonometry inside the
  passes), `transform` constraints and `along` constraints (bones on a
  `path` attachment, with DragonBones' position, spacing and rotate modes),
  and clips of bone, slot (attachment, colour, offset), deform, draw-order
  (`order`) and IK (weight, bend) keys with events; keys ease linearly,
  hold (`step`) or follow a cubic Bézier (`curve`, solved by bisection),
  and a rotate key turns the shorter way or, with `clockwise n` or
  `counterclockwise n`, that way through n more whole turns. A child
  armature plays its parent's clip by name when it has one, else its own,
  at the parent's time. The cook stores the canonical text in an
  `og.Rig2D` blob, written by hand or imported from a DragonBones 5.5
  export with its atlas (`dragonbones "x_ske.json"`, `atlas "x_tex.json"`:
  skew, colour offsets, zOrder and IK timelines, armature displays, boxes,
  paths and path constraints, every skin, Bézier curves, `clockwise` turns
  and the sine `tweenEasing` as Hermite segments within 1e-7; a frame
  without `tweenEasing` holds). Or the cook rigs it from layered art
  (`parts "x.psd"`, or a folder of PNG parts, and a `template`;
  `cook/autorig.rs`): each layer named for a template part, joints where
  parts overlap near their ends, the template's bones through them,
  two-bone IK on the legs, bending parts as meshes weighted across their
  joints through a half-turning joint bone, the template's clip library
  (idle, walk, run, jump, attack, hit, die, wave, cheer, nod, scaled to the
  rig's leg length), and the parts packed into `<rig>#atlas`; a layer
  Photoshop would show otherwise (effects, blend modes, clipping,
  adjustments) or that fits no part is refused, naming the layer, the file
  and the fix. Spine's format is not imported (its licence); `rig2d.rs`'s
  module docs give the mapping a converter follows. `og.load.rig2d`
  registers a rig and its texture (`og.anim.rig2dInfo`, `og.anim.clip`).
  `og.Skeleton2D` (`rig`, `clip`, `time`, `speed`, `loop`, `flipX`,
  `layer`, `skin`, `finished`) plays it through the `og.skeleton2d` system,
  run with the tweens and flipbooks after the game's systems; clip events
  arrive as `og.Skeleton2DEvent`; `og.IkTarget2D` moves a constraint's
  target and reads back `reached` and `error`;
  `og.anim.skeleton2dBoxes` answers the last pose's bounding boxes. The
  pose is a function of the component, its Transform and its IK targets,
  through arithmetic-only sincos and atan2, the same bits on both hosts;
  playback position and boxes are the state block. Each skeleton draws as
  one `og_render::SpriteMesh` (`sprite_mesh.rs`), sorted with its layer's
  sprites and lit by `og.Lighting2D`.

### 2.8 Platform SDK as host ABI

The engine imports a small set of host functions, each a capability the shell
grants: `og_net_fetch` (allowlisted, game-scoped token), `og_store_*` (cloud
save), `og_leaderboard_*`, `og_room_*` (rooms/RTC via the shell),
`og_telemetry`, `og_point` (Origin Point), `og_ai_mark`. In TypeScript they
appear under `og.platform.*`. Nothing in game code can reach cookies, the
DOM or arbitrary origins; the shell owns every credential.

**Rooms.** `og.platform.roomCreate`, `roomJoin`, `roomLeave`, `roomSend`,
`roomPeers`, `roomInfo` and `roomMessage` put this engine with other
players' engines. The engine never opens a socket: it queues ops and its
host carries them through `og_room_take` (the queued ops, JSON),
`og_room_deliver` (one event in) and `og_room_set_host` (`relay`, `player`
or `none`). The wire shapes are og-net's protocol
(`crates/og-net/src/protocol.rs`). Ops: `create`, `join`, `quick`, `leave`,
`send`. Events: `joined`, `peer_joined`, `peer_left`, `message`, `left`,
`error`.

- **Quick play.** `og.platform.quickPlay { mode, players, timeoutMs }` (op
  `quick`) is matchmaking: once `players` of the same game, mode and count
  wait, the host puts them in a room of their own, each hearing `joined`; a
  wait past `timeoutMs` ends in `error` `timeout`; a `leave` takes the
  engine out of the queue (og.RoomJoined `cancelled`). Natively `og relay`
  keeps the queue; on the web the platform's matchmaking does (platform ask
  21).
- **Peers.** A room numbers its peers in join order from 0 and never reuses
  a number.
- **Ordering.** Messages from one sender arrive in the order sent, and its
  departure after its last message. Lockstep's agreement on a leaver's last
  input rests on this.
- **Events in the tick.** Events arrive at the start of a tick, like host
  input, as og.RoomJoined, og.RoomPeerJoined, og.RoomPeerLeft and
  og.RoomMessage.
- **The engine's own channel.** A channel starting with `og.` is the
  engine's (lockstep's `og.lockstep`); its messages never reach the game.
- **Limits.** A game message is at most 64 KiB; an engine message at most
  16 MiB (a snapshot for a late joiner).

Each host carries rooms its own way:

- **Natively.** `og relay [--port P] [--host <ip>]` (default port 47900)
  serves rooms by URL path. `og run --room ws://127.0.0.1:P/<room>`
  connects through og-net's client on a thread of its own and joins
  `<room>` before the first tick; the run pumps the link around every
  frame. `--latency MS --jitter MS --loss PCT --seed N` make the relay a bad
  network (every frame held, unreliable sends dropped and reordered).
- **On the web.** The Worker posts the engine's ops to the page and delivers
  the page's events; the shell's `RoomHost` maps them onto the Player's
  `room.*` (docs/platform-asks.md #16). A Player whose hello does not list
  `room.join` leaves the host as `none`, and roomCreate and roomJoin are
  refused. A Play link to a room (`?room=<id>`) is handed to the engine with
  the host (`og_room_set_host`'s `join`): the engine joins it itself, once,
  as soon as a host carries rooms, so the game hears the same og.RoomJoined
  as for its own roomJoin, and roomInfo's `link` names it. On a page with
  no room host og.RoomJoined says so (ok false). The Worker's `advance` and
  `frame` calls pump the room, and its `net` call answers og.net's status
  and shared hash (`packages/engine/tests/web-rooms.test.ts`).

**Purchases, ads, achievements, identity and share**
(docs/platform-asks.md #19). `og.platform.products`, `purchase`,
`entitlements`, `consume`, `adReady`, `showAd`, `achievementUnlock`,
`achievementProgress`, `achievements`, `player`, `friends`, `inviteSend`,
`invites` and `share` are requests on the same channel, answered with
game-scoped opaque ids; accounts and money stay the platform's.
`og.platform.supports` asks whether the host answers a method; one it lacks
is `unavailable`.

- **Notices.** Unasked host input (a pending purchase settled, entitlements
  changed, an invite) comes in through `og_platform_notify` and is applied
  at a tick's start as `og.PlatformNotice`, logged with the answers.
- **Grant once.** `og.platform.grant` records a delivered purchase in
  `og.PlatformGrants`, world state, so a consumable is delivered once across
  restores, replays and relaunches. Entitlements are always the host's
  answer.
- **Ads.** The host pauses the game through its lifecycle while an ad shows
  and answers after the resume.
- **Sessions.** In a running lockstep or rollback session a system that
  writes shared state hears no platform answer or notice: they are this
  peer's alone.
- **Replay.** `og replay` keeps every answer and notice with its tick and
  feeds them back on compare.
- **The fake.** Natively and in playtests a deterministic fake answers
  (`og run --platform-fake`, `PlaytestDesc.platform`); a published page
  never fakes them.

### 2.9 The video producer

A video is a command script played headless and recorded, built on the fixed
tick, exact seeking by snapshot, headless runs and the offline mixer
(§10.3).

- **Frames.** `og render [project] --commands "..." --fps F --from S --to S
  --size WxH` runs the game on the native host at its own tick and draws
  the world once every `rate / F` ticks (F must divide the tick rate;
  otherwise the error names the nearest that do), so frame i is the world
  at tick `round(S · rate) + i · rate / F`, never between ticks. Ticks
  between frames are simulated without drawing. Each frame is read back
  through og-probe's `Capture::frame` (the frame the loop drew, no second
  draw) and streamed to `og` as RGBA rows
  (`packages/engine/src/node/render.ts`), which writes PNG or JPEG frames,
  pipes them to ffmpeg, and hashes them.
- **Audio.** `--audio` runs the capture mixer from tick 0 to the range's end
  into a 16-bit WAV, muxed into the video (AAC in `.mp4`, Opus in `.webm`);
  rendered from the tick's command stream, it is the same bytes every run.
- **Encoding.** ffmpeg is found through `OG_FFMPEG` or `PATH` and reported by
  `og doctor`; `.mp4` is H.264 (yuv420p, CRF 18) and `.webm` VP9 (CRF 32).
  Without ffmpeg the render writes its frames and says so.
- **Determinism as a contract.** `frames.json` lists every frame's tick,
  time and the SHA-256 of its RGBA bytes (before encoding) and the audio's
  SHA-256; two runs of one script give the same list
  (`packages/engine/tests/og-render.test.ts`). A changed hash is a changed
  frame.
- **Jobs.** `--jobs N` splits the range in N parts. A leader simulates the
  script without drawing and writes a render snapshot (og-probe's
  `render_snapshot`: the world exactly, input resources and the probe's
  pointer included) eight frames before each part; each part restores its
  snapshot, drops the steps the leader ran, draws those eight frames unsent
  (TAA and adapted exposure settle) and renders its frames; ffmpeg
  concatenates the parts. The frames equal a one-job render's.
- **The review surface.** `--contact-sheet COLS` writes a grid PNG of frames
  spaced over the range (COLS columns, ceil(COLS / 2) rows), each labelled
  with its time and tick. `--timeline <name>` is `--commands
  "0:timeline:play:<name>"` with the timeline's `duration` as the range
  (§2.7).
- **Footage.** A `type video` asset (`.ogt` naming an `.mp4`, `.webm` or
  `.mov`, with `fps`, `maxSeconds`, `size`, `kind`, `audio`) is cooked by
  ffmpeg (ffprobe plans it) into an `og.Video` blob, a texture array of its
  frames (`<id>#frames`, each a KTX2 layer of one mip, at most 256) and its
  sound (`<id>#audio`, QOA); the cook reports the bytes and warns past 64
  MiB. `og.load.video` makes one Texture of a frame's size (og-render's
  `FrameTexture`) whose contents become the frame shown, so a material map,
  an `og.Sprite` or a UI image draws footage at one frame's GPU memory.
  Each further player gets a view of its own over the same frames
  (`VideoPlayer.texture`). `og.VideoPlayer` (`video`, `playing`, `time`,
  `speed`, `loop`) is played by the `og.video` system by the tick (not the
  time scale) and keeps the entity's `og.AudioSource` in step: its
  `startOffset` is the video's time, and a seek stops it for a tick and
  starts it again there. A restored snapshot plays on as the world that
  took it (`packages/engine/tests/og-video.test.ts`).
- **Streams.** `stream true` cooks the source to AV1 (8-bit 4:2:0, BT.709
  limited range; ffmpeg's libsvtav1, else libaom-av1; a key frame every
  `keyframe` seconds, one temporal unit per frame), read from ffmpeg's IVF
  into the engine's `og.VideoStream` blob (`<id>#stream`: the sequence
  header, a frame table of offset, size and key, the units), so no
  container is parsed at run time. Beside it, `<id>#poster` (frame 0 as
  rav1d decodes it) and `<id>#audio` in Opus. The stream is not in the
  startup bundle: `og.load.video` reads (on the web, fetches) it. Each
  player draws into a stream texture (og-render's `add_stream_texture`,
  written in place). Natively each player has a rav1d thread
  (`crates/og-video`; rav1d is BSD-2-Clause) that decodes ahead and from
  the key frame at or before a seek; the web decodes with WebCodecs in the
  engine Worker, or with rav1d built for wasm in a Worker of its own where
  the browser has no AV1 decoder (`dist/av1/og-av1.wasm`, fetched only
  then, never in the engine's wasm or a page's boot). A stream larger than
  1280x720 (720x1280 portrait) also cooks a 720p rendition
  (`<id>#stream720`, `og_video::stream::fallback_size`), encoded 6 crf
  coarser (`crf720` overrides), which the fallback plays. A player fetches
  only one, and a publish build's service worker precaches neither.
  `og.video.capabilities` reports the decoder (`rav1d-wasm`, hardware
  false, max 1280x720).
  **Time:** `VideoPlayer.time` is world state advanced by the tick (dt ×
  speed), frame = floor(time × fps); the sound is the AudioSource started
  at `startOffset` = time, so the tick is the one clock. `time` is an f64
  summed with og.audio's tick length, as the sound's playhead is; the cook
  pads the sound with silence to the picture's length; the loop region
  ends at the picture's end (`loopEnd` = frames / fps). og.VideoPlayer is
  at version 2 for the f64 `time` (`save_migrations` widens a version 1
  save) and lazy. Pausing is `playing` or the game's ticks stopping; the
  time scale and hit stop leave footage alone. Several players of one
  stream each have their own decoder. Decoding is presentation. An
  executor (`og run`, `og render`, `og test`) waits for each player's
  exact frame before drawing, so captures are deterministic; a browser
  shows the newest frame decoded and keeps it while the next is late.
- `og render` and its capture members are development members, absent from
  `publish` builds. Agents iterate at a small size and review the contact
  sheet.

### 2.10 Lockstep

Deterministic play over a room (IDL module `net`, crate `og-net`). Every peer
runs the whole world; only the players' inputs and the world's hash cross
the room. It rests on §2.4's determinism and snapshot.

- **Settings.** `og.Lockstep` holds `enabled`, `players`, `inputDelay` and
  `hashEvery`, set by the game; the engine sets `localPlayer` and
  `running`. It belongs to this peer's session: never hashed or compared.
- **Inputs.** `og.PlayerInput` is a reflected component: `player` (the
  seat), `buttons`, `x`, `y`, `aimX`, `aimY` and `tick`, one entity per
  player. At the start of each tick the engine writes every field except
  `player` from that seat's input for the tick (zero for a seat with
  none). The input's packed bytes, read through the component's layout,
  cross the room. A game submits its own input with `og.net.submit`, called
  from the only system that reads `og.Input`; it acts `inputDelay` ticks
  later. Every other system reads `og.PlayerInput`. Without a room, or with
  lockstep off, a submit drives the local player on the next tick.
- **Input shapes and commands.** `og.net.shape` (outside a system, before
  the room starts, the same on every peer) replaces `og.PlayerInput` with a
  game's own component (a u32 `player`; a u32 `tick` written when present;
  `og.net.submitInput` gives it) and declares a command record: orders that
  act on one tick (`og.net.command`, at most `maxCommands` per seat and
  tick). A seat's input for a tick is its state's packed bytes followed by
  that tick's commands; `og.net.commands` reads them on every peer, by seat
  then in order, `player` set to the giver's seat. A prediction repeats a
  seat's last state and never its commands; a command given while a tick
  is re-simulated is dropped. The start message carries the shape's
  fingerprint (its records' names and field types); a peer whose shape
  differs refuses it. Lockstep's wire version is 4.
- **Start.** Until the room holds `players` peers, each world runs freely
  (the lobby). Then the authority (the lowest peer number) starts lockstep
  with its settings; every other peer restores the authority's snapshot,
  and seats go to peers in number order. Once the room is full, every
  other peer holds its world until that snapshot arrives (at most
  `stallTimeout`, 5 s when 0). Lockstep messages arriving after a `start`
  and before it is applied act once its run began, in order.
- **The tick gate.** Tick t runs only when every seated player's input for
  t is in. The gate is og-probe's (`set_gate`, consulted by step, frame and
  advance); a frame that cannot run a tick runs none.
- **Rollback.** With `og.Lockstep.mode` `rollback` (the authority's is the
  room's), lockstep is prediction depth 0: a tick whose inputs are not all
  in runs on each missing player's last input, up to `maxPrediction` ticks
  (and no more than the peer can re-simulate in half a frame) past the last
  confirmed tick. The engine keeps a snapshot of every tick not yet final
  in a ring; when a real input differs from its prediction, the gate
  rewinds to that tick's snapshot (`save::rewind`, crate-internal: no
  check, migration or `og.WorldRestored`) and runs the ticks again in the
  same frame. A tick is final once it ran on real inputs only; the
  compared hashes and `og.net.finalHash` cover final ticks alone, so the
  confirmed world equals lockstep's for the same inputs. Presentation is
  deduplicated outside the tick: a re-simulated tick replays each sound
  once (a new play kept only if at most 100 ms late), the view snaps, and
  `og.net.resimulating` tells a game's own effects to hold.
- **Stall, drop and rejoin.** A running peer silent for a second is reported
  (`og.PeerStalled`, session-only, and the status's `stalled`). Silent past
  `og.Lockstep.stallTimeout` (5000 ms; 0 never drops), or gone from the
  room, its seat is dropped: the acting authority (the lowest running peer
  not itself silent) asks every other running peer how far it holds the
  seat's inputs, each stops taking more, and the seat empties from the
  tick after the furthest, with the inputs up to it handed to all. So
  `og.SeatDropped { seat, peer, tick, left }` arrives at one tick on every
  peer, and the seat's input is empty from then on (a game may hand it to a
  bot). Wall time decides only when a peer is reported and dropped. A
  dropped peer that comes back holds its world and asks again; with
  `rejoin` (default) the authority gives it back its seat (or a newcomer of
  the same name gets it) from a snapshot, from a tick past every input the
  authority has sent, and `og.SeatRejoined` arrives at one tick everywhere.
  Both seat events are hashed and lazily registered. Each input message
  names the seat changes its sender knows, so one that overtook an `admit`
  or a `drop` waits for the resend.
- **Late join.** A peer that joins a running room asks for a snapshot; a
  running peer sends one, and the new peer takes a free seat (or watches)
  from the tick named in the `admit` message.
- **Hashes.** Every tick each peer hashes its world without its own session
  (the input resources, `og.Lockstep`, input, room, platform, asset and
  stream events, the UI's focus): `og.net.hash` and `og run --json`'s
  `netHash`. Every `hashEvery` ticks a peer sends the hashes since its last
  batch, so the first divergent tick is exact. On a mismatch,
  `og.Desync { tick, peer, peers }` is sent once, and both peers exchange
  their snapshots of that tick.
- **The desync dump.** The probe's `desync` dump (`dump_desync`) names the
  tick, both hashes and the component-level diff: per entity (by name),
  component and field, the local and remote values, plus resources,
  events, blocks and entities present on one side only
  (`packages/engine/tests/rooms.test.ts`).
- **Scripts in a room.** A playtest with `PlaytestDesc.room` runs several
  native processes through a relay it starts. A script's steps sit on the
  room's shared ticks once lockstep runs (the gate's snapshot restore leaves
  the schedule on them, unlike a game's own restore), so a `quit` or an
  `expect` lands on one tick for every peer, and a `then` step relaunches
  every place into a room of its own.

### 2.11 The model review

A model built as code from a reference image is judged
pass by pass on one sheet. The engine does the mechanical half of every
judgment.

- **Fixed cameras.** The command channel's `view:<entity>:front|back|side|top`
  frames the entity's drawn bounds (with its og.Parent children)
  orthographically from +Z, -Z, +X and +Y (the top with the front at the
  bottom), and `view:<entity>:orbit:<degrees>` in a 30° perspective from
  20° above the horizon, a tenth of the frame free on each side, until
  `view:off`. `turntable:<entity>:<n>` queues n orbit views 360/n apart with
  a screenshot each for the next tick's start, then the view before
  (`crates/og-runtime/src/review.rs`). The view is render state: it replaces
  the camera extraction made from og.Camera, which is never written, so no
  snapshot, replay or hash sees it. A skinned model's posed parts are not
  framed. Not built into publish.
- **The numbers and the sheet.** `og compare <reference> <render>`
  (`crates/og-cli/src/compare.rs`) keys each silhouette (alpha where an
  image has it, else ΔE00 above `--key` from the median border colour),
  prints the silhouette IoU and the CIEDE2000 difference (sRGB, D65) over
  the pixels both cover, mean and 95th percentile, per region of a flat
  colour mask or over the overlap, and writes one PNG: the reference, the
  render as compared, the silhouettes overlaid, labelled. The render is
  compared in the reference's frame (resampled when the shapes agree,
  refused otherwise; `--align` matches the silhouette boxes). Two runs give
  the same numbers and bytes.

## 3. API surface

### 3.1 One IDL, two hosts

The engine's public API is declared once (`crates/og-api/api/<module>.ogt`,
the same text format): modules, functions, desc structs, enums, events, with
docs and ranges. From it the build generates:

- Rust trait impls checked at compile time (`og-host-web`, `og-host-node`),
- the TypeScript `.d.ts` and the runtime binding tables,
- the wiki pages (`packages/knowledge/wiki/`), one page per module, with the
  signatures quoted from the IDL,
- the JSON schema the platform's knowledge package consumes.

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
every failure is a returned `Result`-like value with a reason string; no
global mutable state in game code (the world and resources are passed in);
`Vector3`, `Quaternion`, `Matrix4`, `Color`, transform fields, material
parameter names and `og.load.gltf()` follow three.js naming, while the ECS,
the tick and determinism are the engine's.

**The math layer**: three.js's methods as pure functions over the plain
structs, declared in the IDL's `math` module (`host false`: no binding
symbol) and implemented in TypeScript (`packages/engine/src/math.ts`),
exported as `vec3`, `vec2`, `quat`, `mat4`, `box3`, `sphere`, `plane`,
`frustum`, `ray` and `color`: `vec3.add, sub, scale, dot, cross, length,
normalize, lerp, distance, applyQuaternion` (and `vec2` likewise, with
`rotateAround`), `quat.multiply, slerp, fromEuler, invert, rotate`,
`mat4.multiply, invert, transformPoint, transformDirection, compose`,
`Box3`, `Sphere`, `Plane`, `Frustum` (`containsPoint`, `intersectsBox`, from
the camera), `Ray` (`intersectBox`, `intersectSphere`, `intersectPlane`,
`intersectTriangle`, null on a miss) and `color.fromHex, toHex, lerp,
fromHsl` (hex and HSL are sRGB, the result linear). No classes and no
mutation of arguments, so a result can be written straight into a column.
Only `+ - * /` and `sqrt` (sincos as the camera systems'), so a system using
them stays deterministic. `packages/engine/tests/math.test.ts` holds cases
against three.js's documented results.

### 3.3 Command channel

`og run --commands "t:action[:args];..."` and the probe's `commands(script)`
schedule actions on sim time. Core actions ship with the engine; a module or
a game registers more (`commands:` above). Unknown actions warn and continue.

| Action | Does |
|---|---|
| `screenshot[:path][:full\|:N]` | capture a frame (readback, next frame) |
| `touch_move_to:<ui id>` / `touch_move:x,y` / `touch_start` / `touch_end` / `scroll:dx,dy` | pointer by UI id or logical coordinates |
| `key_down:<Key>` / `key_up:<Key>` / `type:<text>` | keyboard, display names |
| `move_to_text:[=]text[@N]` | pointer to a UI element by its text |
| `teleport:<entity>:x,y,z` / `frame_entity:<entity>` | place and look |
| `dump_world[:path]` / `dump_entity:<name>[:path]` / `dump_ui[:path]` | tables, systems and their matched counts; one entity's components and transform; the UI tree with bounds |
| `dump_memory[:path]` | resident (native) or linear (web) memory, the counted heap, entities, the stream budget, the audio bank, GPU memory by textures, meshes, render targets and buffers |
| `set:<entity>.<component>.<field>=<value>` | a Tune write through the schema |
| `seed:<n>` / `pause` / `resume` / `advance:<ticks>` | determinism controls |
| `describe[:path]` | the frame as data (§3.4) |
| `expect:<operation>` | an assertion at this tick (§3.4); a failure fails the run |
| `save_scene[:path][:spawned]` | every scene's `.ogt` rewritten from the world, resources included; with `spawned`, the run-time entities under a marked section (§3.4) |
| `quit` | exit |

### 3.4 Probe

The engine implements `RuntimeProbe` protocol 5 natively (`hello`, `tick`,
`state`, `advance`, `pause`, `resume`, `input`, `accepted`, `pick`,
`entity`, `bounds`, `patch`, `run`) plus the wire-only members `capture`,
`sweep`, `reload`, `snapshot`, `restore`, `dump`, `commands`. Transport:
in-page calls from the shell (through a request ring to the Worker, no
postMessage hop), a loopback WebSocket from `og` natively. `publish` builds
keep the player-facing members (`hello`, `tick`, `state`, `pick`, `entity`,
`input`, `capture`) and compile out the rest with what only they read
(`describe` and the views' id passes, `save_scene`'s writer, `reload`, the
modules' dumps). Their command channel keeps the actions og.ai's commands
and a timeline's event keys reach (input, `set`, `teleport`, the clock,
registered commands) and refuses with a warning the ones that only observe
(`screenshot`, `describe`, `expect`, `get`, `dump_*`, `help`, `listen`,
`save_scene`) (crates/og-probe/README.md §Builds).

Above capture sits the layer that lets an agent check a frame without
reading pixels:

- **The semantic frame.** `describe` (member and action) answers the current
  tick's frame as JSON, deterministic for the same world: the visible
  entities nearest first, each with its name, archetype, label,
  screen-space box and pick point (CSS pixels), depth and the fraction
  others hide (objects in front, and UI elements drawn with an opaque
  background over it); the UI elements with their rects and text (from
  `dump_ui`); the camera. The entities come from an entity-id pass
  (og-render's `id_pass.rs`) that renders the extracted frame (meshes,
  skinned meshes posed on the CPU, world sprites) at half width and height
  on the CPU, so a host without a GPU describes the same frame. `pick`
  answers from the same pass when the game has no picker, so a box from
  `describe` contains the point `pick` names.
- **What the secondary cameras show.** The id pass runs again for each
  og.CameraView and og.Mirror, from its eye and on its layers, at a quarter
  of its image's pixels. `describe` lists the views (`views`: id, kind,
  image size and its entities, boxes in the image's pixels), and every
  main-camera entity names the views that also show it. An entity on a
  layer only a view admits is found under its view: visibility is per
  camera, never merged. `expect:entity:...:within` and `pick` read the main
  camera.
- **Expectations.** `expect` operations run as actions (`expect:<op>`) and as
  a probe program step, evaluated at their tick:
  `entity:<id>:within:x,y,w,h` (its described box inside a screen box) or
  `within:x0,y0,z0,x1,y1,z1` (its position inside a world box),
  `event:<Name>[:count]`, `ui:text:[=]<text>`,
  `region:x,y,w,h:color:#rrggbb[:tol]`, `frame:<ref.png>:rmse:<=v`,
  `state:<path><op><value>` over the game's `state()` facts,
  `entity:<id>:<path><op><value>` over a field of `dump_entity`
  (`render.lod==2`) and `timings:<path><op><value>` over `state().timings`
  (`triangles<=1500000`). Each result keeps its tick, what was expected and
  what was observed; a failure is a warning in the log. `og run --json`
  carries them (`expectations`) and exits 1 on any failure; a playtest may
  be expectations alone. A frame expectation without a renderer is
  unchecked, never a failure.
- **Trace.** `og run --trace out.json` writes the run in the Chrome trace
  format (Perfetto opens it; `packages/engine/src/trace.ts` builds it on
  both hosts); every span carries the start it was timed at. Threads:
  engine (each tick, its systems inside it), render (each frame, its
  render-graph passes' CPU encoding), gpu (the same passes from timestamp
  queries where the device has them and the build is not `publish`,
  `RenderCapabilities.gpuTimestamps`; each frame anchored at its submit),
  load (each cooked output read), cook and `cook N` (each output on the
  cook thread that made it), and `lane N` (each og-lanes job on pool thread
  N, with its site). The live probe's `state().timings` carries the last
  tick's `systems[]`, the last frame's `passes[]` and `gpuPasses[]` of the
  newest frame read back (name, ms). On the web the engine Worker records
  the same document (the play page's `?trace` from the boot, or the probe's
  `trace("start")`) and the shell reads it (`trace()`, `saveTrace()`
  downloads it).
- **Scene write-back.** `save_scene` (member, and
  `save_scene[:path][:spawned]`) rewrites every scene the world was
  assembled from, each to its own `.ogt` (the boot scene's, then each
  `og.world.instantiate` made), line by line: a field line changes only when
  its value, rounded as the cook rounds it, differs; a field `set` or
  `patch` tuned that the text lacks gets a line (or a component block);
  comments, order and spelling stay. Resources are scene text too
  (`resource <Name> { field value }` at the top level sets those fields when
  the scene is instantiated): a scene's resource lines are rewritten, and a
  tuned resource no scene sets gets a block in the boot scene. With
  `spawned`, the entities the game spawned at run time are written at the
  end of the boot scene under the line `# ── spawned at run time ──`,
  regenerated whole on every save, nested by og.Parent, each with every
  component scene text can give (a model's `model` line; a component holding
  a geometry, material, sound or entity handle is named in a comment, not
  written; an instance's root is left out). Cooking the result loads the
  same world, and saving again changes nothing
  (`packages/engine/tests/scene-write-back.test.ts`).
- **Diagnostics as data.** `og cook --report` writes
  `og-out/cook-report.json`. For each asset it records triangles, vertices
  and LODs, textures with sizes and formats, bones, clips, bytes cooked and
  budget flags, and every other output type its own figures, read back
  through the runtime's loaders (a sound's seconds, channels, rate and
  chunks; a font's glyphs, kerning pairs and atlas pages; a scene's
  entities, instances and components; a navmesh's tiles, polygons and
  vertices; a timeline's tracks and seconds; a string table's locale and
  keys; and likewise styles, behaviours, tilesets, tilemaps, rigs,
  flipbooks, shaders, post passes, emitters, looks, palettes, LUTs,
  environment maps, videos, terrains and stream packs; a generated terrain
  its height histogram, land above the sea and river length, with its
  height and control maps as PNGs beside the report). `og doctor --project
  <dir> [--json]` checks it against the budgets project.ogt sets (`budgets
  { triangles vertices textureSize bones clips assetBytes totalBytes
  soundSeconds videoSeconds fontGlyphs navPolygons sceneEntities
  emitterParticles }`, each with a default) and prints findings with
  severity, asset, budget, limit, value and fix. `bvhShare` (30) is a
  warning, not a limit: a model whose cooked raycast BVH is over that
  percent of its blob and at least 16 KiB is named, with `model { raycast
  false }` as the fix. Source material warnings also name full-strength
  `[1, 1, 1]` emissive factors with an emissive texture whose mean linear
  Rec.709 luminance is >0.01 (alpha ignored): `emissiveTexture` warnings
  name the asset, material and mean so an author can check for albedo
  baked into emission. They persist on cached cooks and never modify the
  texture or material.

### 3.5 Hot reload

`reload` snapshots the world, swaps the game bundle (bun rebuilds in tens of
milliseconds), re-registers components by name and restores. A component
whose layout changed is migrated field by field through the schema; a
removed one is dropped with a warning in the reload report.

The engine's own Rust takes the same path one level down, for engine
developers in a checkout (`og dev --engine`; a game author never runs it):
a change to a crate rebuilds the dev addon while the game keeps running,
then the game hands its world over (`run.ts --handoff`: the save written
between two frames on SIGUSR2) and a new process on the new addon restores
it (`--restore`, migrating by name) and goes on from that tick. It is a
restart, not a patch of running code: determinism and the whole-world
snapshot make the restart lose nothing. A failed build keeps the old game; a
change the addon does not link restarts nothing.

### 3.6 The inspector and gizmos

A development build carries an inspector for humans (og-runtime's
`inspector` module, compiled out of `publish`): F1 shows it (Ctrl+F1 when
the game binds F1; `inspect:key:<code>` sets another), over every layer of
the game's frame, in the engine's default style. It reads only the
reflected schema and writes only through the probe:

- **Panel.** Entities by og.Parent and the world's resources, filtered by
  name; the selected one's fields, each with the editor its type and column
  kind ask for. Every edit is the probe's `set` (`Probe::tune`): checked
  against the schema, applied between ticks, kept for undo (a drag folded
  into one step), marked unsaved, and flagged when the game writes the value
  back. Save is `save_scene`, which writes each tuned field into its line of
  the scene text; a resource's field into its `resource` block.
- **Prefab overrides.** An entity placed from a prefab shows its prefab and
  node, and what its instance overrides (the scene's override lines and the
  session's unsaved edits, tinted). Each override, or all of them, is
  applied into the prefab's file (`apply_prefab`) or reverted
  (`revert_prefab`); either clears the fields' unsaved marks.
- **Where Save goes.** Natively the engine writes the files. A web page
  writes none: the save answers the texts, the frame answer carries them
  (`saved: {scenes: [{source, text}]}`), and the shell sends them to a Stage
  whose hello lists `scene_saved` as that event on the og-probe wire (at
  most 64 scenes an event; a scene outside `assets/`, not `.ogt` or past 1
  MiB is not sent) and downloads each one otherwise (`shell/wire.ts`
  `routeSavedScenes`; `docs/platform-asks.md` #22). A prefab's Apply and
  Revert work only where the engine reads the project's files.
- **Gizmos.** On the selected entity's og.Transform: translate (an arrow per
  axis, a square per plane, the centre in the view's plane), rotate (a ring
  per axis and one facing the view) and scale (a bar per own axis, the
  centre uniformly). W, E and R pick the mode (over the panel, or anywhere
  the game binds none of them), translate and rotate follow the world's or
  the entity's axes, and the handles keep one length on screen (96 px). A
  drag casts the pointer's ray from og.Camera against the handle's line or
  plane (a ring seen edge on turns by travel along its tangent), converts
  the world pose to the parent's space, and writes one field a frame through
  the same `set`, folded into one undo step; Ctrl snaps a move to 0.5, a
  turn to 15° and a scale to 0.1 of its factor. The selected meshes' bounds
  box draws with it.
- **Observable.** `dump_inspector` answers the panel's elements with their
  rectangles and the gizmo's mode, space, hovered and dragged handle and
  each handle's grab point on the surface, so a script presses what a person
  would; `inspect:mode:<m>` and `inspect:space:<s>` set the gizmo.

## 4. The play surface without iframes

The platform serves a top-level Player on the play origin
(`play.<domain>/g/:slug`, `goals/play-surface` D1, D2), and it is the only
mode:

- Desktop and the creator Stage use the same top-level page. The Portal
  opens play in the same tab with the one-time identity handoff; the creator
  overlay (chat, Stage controls, Tune, annotations) renders in the play page
  itself, talking to Agent Server over its thread socket with the scoped
  delegation the handoff mints.
- The play origin carries no Portal cookie; the shell holds a game-scoped
  player token. Game JavaScript runs in the page, which is acceptable
  because the page has nothing beyond the game's own scope to steal; the
  engine's host ABI is the only route to platform APIs.
- COOP/COEP on the play page enables lanes, declared per publication.
- The shell is the brand surface: boot splash (`genesis` motion), real
  progress (`progress`), ready-to-interact gate, safe areas, `100dvh`, one
  audio unlock, PWA install.

## 5. Executors and the sandbox

| Class | Contents | Image estimate |
|---|---|---|
| `engine` | bun, the engine (wasm + native addon), git | ≈ 400 MB |
| `probe` | the same plus ffmpeg (minimal static build) | ≈ 600 MB |
| `asset` | plus Blender headless (trimmed) and the cook tools | ≈ 1.8 GB |
| `browser-check` (evaluation only) | plus Chrome headless-shell for the final "runs in a real browser" check | ≈ 800 MB |

No Godot, no Chromium in the inner loop, no Rust toolchain anywhere in the
sandbox. An image installs the released version the platform pins: the
signed archive for its target from `/downloads/engine/<version>/`, the same
install an author's machine and the platform's local stack make; never a
layout built from a git revision or an engine checkout (`docs/release.md`
§1). Sweeps, replays, determinism and the probe's dumps run through `og` on
the native addon without a GPU and are deterministic. Frames need a GPU:
where there is no adapter, a playtest's frame checks are reported unchecked
(never passed), and a frame is verified on the author's machine, in
`browser-check`, or on an executor with a GPU. Performance truth comes from
the person's device through Player telemetry and from `og run --device`
(safaridriver / Chrome DevTools on a paired phone, timing the publish build)
when the person is present.

## 6. Failure modes

| Violation | Behaviour |
|---|---|
| stale typed view used after a structural change | throws with the owning tick and the system name |
| dead handle | error value `{ok:false, reason}`; never a trap |
| system writes a component it did not declare | refused at registration with the field name |
| wall-clock read inside a tick | not reachable: `Date`/`performance` are not exposed to systems; the lint flags imports |
| wasm trap (engine bug) | the shell shows the crash card with the last 200 log lines and the world dump; telemetry uploads the report |
| WebGPU unavailable | not supported: the player shell says so instead of starting the engine (§8) |
| cook stamp mismatch at load | refuse with the asset path and the two stamps |
| probe refusal | `false`/`null` exactly as protocol 5; `error` only for malformed calls |

## 7. Budgets

Gated in CI where `scripts/budgets.txt` holds them, measured at a wave's end
otherwise. Quality comes first: the engine targets current devices only
(§8), and no budget is a reason to cut a feature to a reduced tier or hold a
model or an effect below the quality it needs; a figure a feature moves is
pursued, or the budget is raised in its own commit (docs/ci.md §Updating a
budget).

| Budget | Target |
|---|---|
| engine wasm, brotli | ≤ 16 MB per download (the largest variant a player loads): a ceiling against accidental growth, not a figure to cut features for. Model weights and other assets are not in it |
| first frame, a current iPhone, 4G, small game | ≤ 2.0 s click to first frame (the play-surface budget) |
| first frame, iPhone 13, Wi-Fi, mid-size 3D scene | ≤ 1.5 s |
| TS system over 10,000 entities (one archetype, 2 components) | ≤ 1.0 ms per tick on a current iPhone |
| reference scene (`crates/og-tests/reference-scene`: 500 instanced objects, 2 shadowed lights, 20 skinned characters, PBR, post) | reported by `packages/engine/bench/lanes.ts`, native and in Chrome; no Three.js comparison runs (its pages are removed) |
| probe `capture` 1280×720 | ≤ 30 ms native, ≤ 60 ms web |
| hot reload | ≤ 300 ms from bundle write to first new frame |
| headless executor image (`engine`) | ≤ 600 MB |

The hot-reload budget is rebuild and swap. `og dev`'s type check runs beside
the rebuild in a process of its own and is reported apart (`check.ms`); it
never delays or refuses a reload, so it is outside the 300 ms.

The wasm budget is per download: a page downloads one engine module, the
publish variant of its game's physics profile (3D or 2D) or, on a
cross-origin isolated page, the lanes variant, so each module is held to it
and the figure that counts is the largest one a player downloads, never the
sum (docs/web-export.md §Variants). The 2D profile's modules leave the 3D
world out as well as rapier3d (og-render's and og-runtime's `profile2d`:
terrain, planar water, the 3D fluids, scattered copies and volumetric fog;
`RenderCapabilities.world3d` is false, and `og.terrain`'s calls and a
`SHAPE_HEIGHTFIELD` collider answer a refusal naming it), so `og build
--publish` refuses a 2D project whose scenes hold them, by scene file and
line.

## 8. Platform matrix

| | WebGPU (Chrome, Edge, Safari 26+, recent Firefox) | native Linux (executor) | native macOS (developer) | native Windows x64 |
|---|---|---|---|---|
| renderer | full | full with a Vulkan GPU, else none: the simulation, the probe and the playtests without their frames | full (Metal) | full (DX12) |
| lanes | with COOP/COEP | rayon | rayon | rayon |
| audio | AudioWorklet | cpal or null | cpal | cpal |
| capture | readback | readback | readback | readback |
| determinism | yes | yes | yes | yes |

The engine targets current hardware and current browsers; old browsers,
low-end GPUs and old drivers are not a target and are not tested. There is
no reduced tier for weaker devices: a browser or device without WebGPU is
not supported (the player shell says so instead of starting a degraded
engine), and a feature is built for the supported devices at the quality it
needs.

- Browsers: the current stable Chrome and Edge and the major before it,
  Safari 26+, the current stable Firefox, on WebGPU. No WebGL2 fallback.
- Desktop: Windows 10/11 x64 on DX12 (in the browser, D3D12 for WebGPU) with
  a current NVIDIA, AMD or Intel GPU and driver; macOS on Apple silicon
  (Metal); Linux x64 with a Vulkan GPU.
- Phones: a current iPhone in Safari. Mid-range and older Android devices
  are not yet targeted.
- The reference GPUs are real ones: `metal` (macrunner) and DX12 with the
  Windows browsers (studio-win). lavapipe is only the renderer of machines
  without a GPU: a smoke check that a frame draws, never a reference tier.

Windows x64 uses the same native API: the MSVC-built target carries
`og.exe` and `origin-engine.node`; its automatic renderer is DX12, and `og
check` uses the pinned tsgo fallback because tsrs has no Windows package.

## 9. Branding

- **Name**: Origin Engine. The name belongs to this engine; the platform's
  older Godot workspace is "the Godot distribution".
- **Prefix**: `og` everywhere a short identifier is needed: the CLI (`og`),
  crates (`og-*`), formats (`.ogt`, `.ogb`), the host ABI (`og_*`), the
  TypeScript root object (`og`). The platform uses `og-` for credentials,
  `og-probe` for the wire and `OG_*` for environment, so the family stays
  consistent.
- **Packages**: `@origingame/engine` (the knowledge inside it as
  `knowledge/`), its per-platform native packages (`-darwin-arm64`,
  `-linux-x64`, `-win32-x64`) and `@origingame/create-game`, released as
  one signed archive per target (`docs/release.md` §1). Repository
  `originai-game/origin-engine`.
- **Artifacts**: `origin-engine.wasm`, `origin-engine.node`; telemetry and
  user-agent token `OriginEngine/<version>`; `hello.engine` reports
  `origin-engine <version>`.
- **Marks**: the boot splash is the master symbol in white on ink `#14161b`
  playing `genesis` once; loading uses `progress` with a real value; the
  "Made with Origin Engine" badge in the player shell and the exported
  page's footer. The product mark `engine` in `packages/brand` follows the
  product-mark rule (compositions of the master's parts): the core ring with
  the symbol's two **inner** arms (the engine is what turns inside the
  shell; `kit` is the two outer arms around the core). Geometry lives in
  `packages/ui-svelte/src/lib/brand/product-marks.ts`, never redrawn
  elsewhere.
- **Game UI is not brand, and brand is not game UI**: a game's UI style
  (og-ui, a look's `ui` line) styles that game's own screens, and nothing of
  the platform's design language reaches them: no token, no radius, no
  palette. Only the engine's marks, the boot splash, the loading state and
  the player shell (Back, "Made with") use the platform ink and tokens.
- **Voice**: documentation, the wiki and `og` output use the platform's
  editorial voice (`origingame/goals/editorial-design`): plain sentences, no
  exclamation marks, every error names the fix.
- **Versioning**: semver; a Game pins its engine version in `project.ogt`;
  upgrades are explicit threads.
- **Licence**: MIT (`LICENSE`).

## 10. Depth features: rendering depth, the sequencer, the video producer, rooms and probe depth

The feature areas beyond the core engine that close the gaps against
Babylon, Godot 4 and Unity: runtime geometry, the timeline, the video
producer, lockstep rooms, the probe's semantic layer, LOD and reflections,
baked lighting, render targets and custom shaders, 2D skeletal animation
and the math layer. Each area's design is in `docs/design.md` (the section
named below); the IDL (`crates/og-api/api/<module>.ogt`) is the contract.
A type or parameter that has a three.js name takes that name. `debug`,
`expect`, `trace`, `describe`, `save_scene` and the `og render` machinery
are development members, marked so in the IDL and absent from `publish`.

### 10.1 Runtime geometry, lines, points, debug draw, mesh raycast

- `og.render.bufferGeometry(world, desc: BufferGeometryDesc): Result<Geometry>`:
  `positions` (Float32Array, xyz), optional `normals`, `uvs`, `colors`,
  `tangents`, `indices` (Uint32Array), `dynamic bool`; missing normals are
  computed. `og.render.updateGeometry(world, geometry, desc)` rewrites a
  dynamic one in place (same vertex count or a reallocation).
  `og.render.geometryInfo(world, geometry): Result<GeometryInfo>` (counts,
  bounds). A buffer geometry is a `Collider.mesh` through `og.physics.setMesh`
  as a loaded one is.
- Primitives beside box, plane and sphere, with three.js parameters and
  defaults: `cylinderGeometry`, `coneGeometry`, `capsuleGeometry`,
  `torusGeometry`, `ringGeometry`, `circleGeometry`, `tubeGeometry` (along
  points, Catmull-Rom through them; `og.path.sample` can feed it).
- Lines and points, immediate mode as `og.render.sprite`:
  `og.render.lines(world, LinesDesc)` (points, `color`, `width` in pixels or
  world units, `depthTest`, `screen bool`, `dashed`) and
  `og.render.points(world, PointsDesc)` (`size`, `color`, `sizeAttenuation`);
  retained as components `og.Lines` and `og.Points` over a geometry handle
  for static sets. `StandardMaterialDesc.wireframe bool`.
- Debug draw: `og.debug.line`, `box`, `sphere`, `arrow`, `axes`, `text`
  (world-anchored label), each with a `duration` in ticks (0 is this frame),
  drawn after post and before the UI so they read in every capture; the
  command channel's `debug:on|off`, `debug:physics` (colliders, contacts,
  joints), `debug:nav` (grid or navmesh, paths), `debug:bounds`, `debug:lod`
  (the `RenderOptions.debugView` enum). Stripped in `publish`.
- `og.render.raycast(world, ray, RaycastGeometryDesc): Result<GeometryHit[]>`
  against meshes (not colliders): a BVH per geometry built at load and on
  update, filtered by `layers` (§10.8; default mask 1).
- Code: og-render `lines.rs`, `points.rs`, `debug.rs`, `geometry/` (the BVH);
  the `render` and `debug` IDL modules. Design: `docs/design.md` §2.6.

### 10.2 Paths and the timeline

Cutscenes as an `.ogt` asset the agent reads and diffs; also the edit
decision list the video producer (§10.3) plays.

- Paths: `og.path.create(world, PathDesc): Result<Path>` (`points`,
  `closed`, `kind` `catmullRom | bezier | linear`, `tension`),
  `og.path.sample(world, path, t): PathSample` (position, tangent, up, at
  arc-length parameter 0..1), `og.path.length`, `og.path.nearest`;
  `og.PathFollower` component (`path`, `speed` units per second, `loop`,
  `align` to the tangent, `offset`), stepped by the tick. `og.CameraRail`
  reads a path handle too (`rail.path`).
- `type timeline` asset (`.ogt`, cooked to `.ogb`; `og.load.timeline`):
  `duration` in seconds, `tracks`, each with `kind` and keys at `time`:
  - `camera`: position and target keys (or `follow <entity name>`,
    `rail <path>`), `fov`, eased; while a timeline with a camera track
    plays, it owns `og.Camera` and hands it back on stop;
  - `transform <entity name>`: position, rotation, scale keys with easing
    (the tween `Ease` enum);
  - `animation <entity name>`: clip, speed, blend-in seconds;
  - `audio`: a sound at a time with volume and bus; `music` with fades;
  - `caption`: text, `style` (a UI text style), `anchor`, from `time` to
    `end`; the cook imports an `.srt` or word-timestamp `.json` into a
    caption track (`captions "narration.srt"`);
  - `event`: a command-channel action at a time (`teleport`, `set`, a
    game's registered command);
  - `transition`: `cut | fade | wipe | dissolve` with duration, over og.ui's
    screen transition;
  - `marker`: named times (`og.TimelineMarker` event when passed).
- `og.Timeline` component (`asset`, `playing`, `time`, `speed`, `loop`),
  `og.timeline.play / pause / seek / stop`, events `og.TimelineStarted`,
  `og.TimelineMarker`, `og.TimelineEnded`. Seeking is exact: keys are
  evaluated at the tick's time, never accumulated, so seeking to t then
  capturing equals playing to t. Command-channel actions
  `timeline:play:<name>`, `timeline:seek:<s>`.
- Code: `og-timeline` (evaluation), og-cook (timeline, `.srt`, paths in
  scenes), the `path` and `timeline` IDL modules. Design: `docs/design.md` §2.7.

### 10.3 The video producer

- `og render [project] --commands "..." | --timeline <name> --fps 30
  --from <s> --to <s> --size WxH [--out dir | out.mp4 | out.webm] [--frames
  png|jpg] [--audio] [--contact-sheet <cols>] [--jobs N] [--json]`: a native
  headless run at the project's tick (the fps must divide the tick rate,
  else an error names the nearest), one readback per frame through the
  capture hooks, frames piped to `ffmpeg` (found by `og doctor`; absent,
  frames only) as H.264 or VP9, the capture mixer's WAV muxed with `--audio`.
  `--jobs N` splits the range: each job restores the snapshot the leader
  took at its first tick and renders its range; `ffmpeg` concatenates.
  `--contact-sheet` writes a grid PNG with timestamps and `frames.json`
  (tick, time, sha256 per frame, the audio's sha). Two runs of one script
  produce byte-identical frames. `--timeline` is
  `--commands "0:timeline:play:<name>"` with the timeline's duration as the range.
- Video in: `type video` asset (`.ogt` naming an `.mp4`, `.webm` or `.mov`,
  `fps`, `maxSeconds`, `size`), cooked by `ffmpeg` into a texture array of
  frames (KTX2; the cook reports the bytes and warns past 64 MiB) and its
  audio as a sound; `og.load.video: Result<Video>`, `og.load.videoInfo`,
  `og.VideoPlayer` component (`video`, `playing`, `time`, `speed`, `loop`)
  whose current frame is the `Texture` of a material map, an `og.Sprite` or
  a UI image through `og.VideoPlayer.texture`, with its sound on an
  `og.AudioSource` kept in step by the tick.
- Code: og-cli `render.rs`, `contact_sheet.rs`; og-cook `video`; the `video`
  IDL module. Design: `docs/design.md` §2.9.

### 10.4 Rooms and deterministic lockstep

- Host ABI and `og.platform.roomCreate(desc)`, `roomJoin(id)`, `roomLeave`,
  `roomSend(bytes | json, to?)`, `roomPeers`, events `og.RoomJoined`,
  `og.RoomPeerJoined`, `og.RoomPeerLeft`, `og.RoomMessage`; answered on the
  web by the shell (the platform's rooms, `docs/platform-asks.md`) and on
  native by `og relay [--port]`, a local WebSocket relay, so two
  `og run --room ws://127.0.0.1:P/<room>` processes play together on one machine.
- `og.Lockstep` resource (`enabled`, `players`, `localPlayer`, `inputDelay`
  ticks, `hashEvery` ticks) and `og.PlayerInput` (a reflected component the
  game declares per player: the engine serialises it, exchanges it through
  the room, and advances tick t only when every player's input for t is in;
  with lockstep on a game reads inputs from the component, never from
  `og.Input`). Late join through a snapshot from a peer; the world hash
  every `hashEvery` ticks; on a mismatch `og.Desync` (tick, peers) and the
  probe's `desync` dump: the first divergent tick and the component-level
  diff of the two snapshots.
- Code: `og-net` (lockstep, relay client, the hash), og-cli `relay`, the
  `platform` and `net` IDL modules. Design: `docs/design.md` §2.8, §2.10.

### 10.5 The semantic frame, assertions, trace, write-back, diagnostics

- `describe` (probe method and `describe:<out>` command): the visible
  entities with screen-space boxes, depth order and occlusion fraction
  (from an entity-id pass at a quarter of the frame), their names,
  archetypes and labels; the UI elements with rects and text (from
  `dump_ui`); the camera. JSON, deterministic.
- `expect` operations in probe programs and as command-channel actions:
  `entity <name> within <box>`, `event <name> [count]`, `ui text "<s>"`,
  `region x,y,w,h color #rrggbb tolerance`, `frame <ref.png> rmse <= v`,
  `state <path> == <value>` (the game's `state()` facts); a failed
  expectation is reported with the tick and the observed value; `og run
  --json` carries them and exits non-zero on any failure.
- `og run --trace out.json`: Chrome trace format, per tick a span per
  system, per render pass, per load and cook event, lanes as threads;
  `ProbeTimings` has `systems[]` and `passes[]` (name, ms) for the live probe.
- Scene write-back: probe `save_scene` and command `save_scene[:path]`:
  the entities that came from the scene and the tuned resources written
  back to the scene's `.ogt` text, line-based, keeping comments and order,
  only the lines whose values changed; load, change, save, load is exact.
- `og cook --report` writes `cook-report.json` (per asset: triangles,
  vertices, LODs, texture sizes and formats, bones, clips, bytes cooked,
  budget flags); `og doctor --project <dir>` reads it with the project's
  budgets (`project.ogt`) and prints structured findings (`--json`).
- Design: `docs/design.md` §3.4.

### 10.6 Mesh LOD, reflection probes, screen-space reflections

- LOD: the cook simplifies every model mesh with meshoptimizer into levels
  (`type model` `lods [1, 0.5, 0.25]` by default, `MSFT_lod` levels kept
  when the glTF has them), `og.Model.lodBias`, `RenderSettings.lodBias`;
  the renderer picks by projected size with hysteresis; `dump_entity`
  shows the level; `debugView lod` colours by level.
- `og.ReflectionProbe` component (box extents, `resolution`, `mode`
  `baked | realtime`, `interval` ticks); box-projected sampling in the lit
  shader blended with the environment map; `og bake probes [project]`
  renders the baked ones on the native host into the project's cook cache,
  loaded when present, baked lazily on first use otherwise (and always on
  the realtime path).
- SSR: `PostEffects.ssr`, `ssrMaxDistance`, `ssrThickness`, `ssrSteps`,
  `ssrFade`; where unavailable it is reported in `RenderCapabilities` and
  probes are the fallback.
- Code: og-cook (lod), og-render `lod.rs`, `probes.rs`, `ssr.rs`, og-cli
  `bake_probes.rs`. Design: `docs/design.md` §2.6.

### 10.7 Baked lighting, irradiance volumes, volumetric fog, area lights, cookies

- `og bake lightmaps [project]`: a CPU path tracer (progressive, native,
  no GPU needed) over the scene's static models and lights, lightmap UVs
  from xatlas, the result as KTX2 per scene, read automatically by
  `og.Model` for static entities (`static true` in the scene line);
  `og.LightVolume` component (box, spacing) baked to spherical harmonics
  that dynamic objects sample. Baked data is loaded when present; otherwise
  the scene renders unbaked and `og doctor --project` says so.
- Volumetric fog: `og.Fog` has `volumetric`, `density`, `scattering`,
  `anisotropy`, `height`; a froxel grid lit by the sun with shadows and by
  the clustered lights (light shafts come out of it); a cheaper tier is
  reported in `RenderCapabilities`.
- Area lights: `og.RectLight` component (width, height, color, intensity,
  two-sided) with LTC shading; `SpotLight.cookie` and `PointLight.cookie`
  (a `Texture`, a cube for point) projected.
- Code: og-render `lightmap.rs`, `volumetric.rs`, `area.rs` (and the lit
  WGSL's lightmap and SH hooks), `og-bake` (the tracer, xatlas), og-cli
  `bake_lightmaps.rs`. Design: `docs/design.md` §2.6.

### 10.8 Render targets, secondary cameras, layers, custom shaders, custom post passes

- `og.render.renderTarget(world, RenderTargetDesc): Result<RenderTarget>`
  (width, height, `format`, `depth`), its colour a `Texture` for any map,
  sprite or UI image; `og.CameraView` component: a secondary camera (the
  `og.Camera` fields) rendering into a target or into a `viewport` rect of
  the frame (split-screen), with `layers` mask, `order`, `clear`, `every`
  ticks; `og.Mesh.layers`, `og.Sprite.layers`, `og.Camera.layers` (three.js
  layers, bit masks, default 1); `og.Mirror` component: a planar reflection
  as a mirrored CameraView into a target the material reads.
- Full custom shaders: `type shader` `mode full` with `vertex` and
  `fragment` entries over a documented binding contract (camera, model,
  skin, params, up to eight textures), `lit` or `unlit`, shadows and the
  prepass drawn through its vertex stage; cooked per backend as the hooks are.
- Custom post passes: `type post` asset (a WGSL fragment over the HDR
  colour, depth, normals, the previous pass, params) and
  `PostEffects.custom` (an ordered list of `Post` handles, params set by
  `og.render.setPostParam`); a pass that fails to compile is reported and
  skipped.
- A tier limit is reported through capabilities, never by a missing member.
- Code: og-render `targets.rs`, `views.rs`, `custom.rs`, `post_custom.rs`;
  og-cook (shader modes, `type post`). Design: `docs/design.md` §2.6.

### 10.9 2D skeletal animation and the math layer

- 2D skeletal: `type rig2d` asset in the engine's text (bones, slots,
  attachments: region, mesh with weights, IK constraints, clips with bone,
  slot and deform keys), cooked from that text or imported from DragonBones
  JSON; `og.Skeleton2D` component (`rig`, `clip`, `time`, `speed`, `loop`,
  `flipX`), `og.Skeleton2DEvent` for clip events, drawn through the sprite
  batch extended to triangles for mesh attachments, lit by `og.Lighting2D`;
  `og.anim.rig2dInfo`. Spine's format is not imported (its licence); the
  text format documents the mapping for a converter.
- The math layer: three.js's methods as pure functions over the plain
  structs, declared in the IDL's `math` module and implemented in
  TypeScript (`packages/engine/src/math.ts`): `vec3.add, sub, scale, dot,
  cross, length, normalize, lerp, distance, applyQuaternion`, `vec2`
  likewise, `quat.multiply, slerp, fromEuler, invert, rotate`,
  `mat4.multiply, invert, transformPoint, transformDirection`, `Box3`,
  `Sphere`, `Plane`, `Frustum` (`containsPoint`, `intersectsBox`, from the
  camera), `Ray` (`intersectBox`, `intersectSphere`, `intersectPlane`,
  `intersectTriangle`), `color.fromHex, toHex, lerp, fromHsl`. No classes:
  data stays plain so columns and reflection are untouched.
- Code: og-anim (2D), og-cook (rig2d, DragonBones). Design:
  `docs/design.md` §2.7 (2D skeletal), §3.2 (the math layer).

## 11. Temporal upscalers: TAAU and FSR 3.1

Under TAA the engine resolves the jittered frame, drawn at the render
scale, to the output with one of two upscalers: `RenderSettings.upscaler`
(`og-render`'s `RenderQuality::upscaler`), `taau` or `fsr3`.
`RenderCapabilities.upscalers` names the ones a device draws: both on every
host and tier.

### 11.1 The two

- **TAAU** (`taa.wgsl`): one pass. The sample nearest each output pixel,
  weighted by its distance, blended into an output-sized history that is
  clamped to the current samples' neighbourhood and resampled with the
  bilinear or Catmull-Rom filter (`RenderOptions.taaHistory`), then
  `taaSharpen`'s pass.
- **FSR 3.1** (`fsr_*.wgsl`, `post/fsr.rs`): AMD FidelityFX Super
  Resolution 3.1.6's upscaler, ported to WGSL (MIT,
  `third_party/fidelityfx/`, whose README lists every deviation): eight
  compute passes for dilated depth and motion with depth-reconstructed
  disocclusion, a luma pyramid and auto exposure, shading-change
  detection, reactivity and thin-feature locks, luma instability, and
  the accumulation (Lanczos-2 upsample and history, variance-box
  rectification in a reversibly tonemapped YCoCg), then RCAS
  (`taaSharpen`'s strength). Transparent surfaces feed its reactive mask
  through the order-independent transparency's coverage.

FSR runs in fp32 without subgroups, float atomics or device-wide
coherence: it needs neither `shader-f16` nor `subgroups`, which Safari
and Firefox lack, so one shader serves every host, at the cost of AMD's
fp16 speed-up where a GPU would have it.

Other web upscalers (pmndrs/upscaler, a cut-down FSR 3.1; three.js
`TAAUNode`, a TAAU of the same class as ours; Arm ASR; Snapdragon GSR 2)
beat neither of the two, so none is ported.

### 11.2 The choice

FSR 3.1 is the better picture whenever the camera or an object moves, and
steadier everywhere; TAAU is cheaper (FSR costs about 1 ms to 2 ms more at
1080p and 1440p), converges faster after a cut and is as good on grass
and particles. So:

- **Default and medium: TAAU.** The default is also every native run's
  resolve at render scale 1 (`og run`, `og test`, the references).
- **High: FSR 3.1** (`og.render.applyQuality(world, "high")`). A game sets
  `RenderSettings.upscaler` itself at any preset.
- **Low: TAAU.**
- **Dynamic resolution:** automatic quality lowers the render scale with
  the upscaler kept (FSR's state is the output's size, so a step keeps its
  history), and once its preset steps cap the quality at medium or low it
  draws TAAU (a cap takes the cheaper upscaler), the history starting
  again at that step.

`fsr::compare` and `fsr::cost` (`crates/og-tests/tests/fsr.rs`, ignored,
run by hand on Metal) measure quality against a 4-samples-a-pixel ground
truth and the frame's cost; their numbers go in reports.

## 13. Alternatives considered

- **Bevy as the base.** Mature wgpu renderer and ECS in Rust, MIT/Apache.
  Rejected because the API changes every release, which defeats the stable
  API the agent's knowledge depends on, its wasm builds are large, and web
  multithreading is not supported. Its crates (wgpu, naga, glam) are used
  directly.
- **Keep Godot, improve the tooling.** The headless footprint, the
  editor-centric data model and GDScript remain; the agent's inner loop
  stays slow.
- **Three.js with an imposed structure.** Keeps the knowledge advantage but
  leaves JavaScript as the per-frame engine and the iframe as the
  isolation; the performance and play-surface goals are unreachable.
- **A C engine like HZ.** Proven shape, but Rust gives the same wasm output
  with memory safety; game code in C would put a compiler back into the
  author's loop.
- **Game code in Rust.** Strongest isolation and performance, rejected for
  compile latency, toolchain size in every sandbox and the author's loop.

## 14. Sources

Observed shapes, not code: the HZ engine's AI-native workflow (command
channel, evidence loop, wiki-as-map, reflection to blobs, one build tool);
wgpu, naga, rapier, meshoptimizer, KTX2/Basis, QOA, MSDF, Clay layout; the
platform's probe protocol 5, its play-surface and agent-system decisions,
and the platform's design language for the player shell and the marks only
(§9).
