# Origin Engine — design

Normative. Decided items are marked **(D)**; open items are listed in §12 and
owned by the platform owner. The structure follows the discipline we observed
working elsewhere: API surface first, then data and lifetime, threading,
failure modes, budget, platform matrix, migration, alternatives, sources.

## 1. Problem and non-goals

**Problem.** OriginGame produces games through an agent that must write, run,
observe and correct a game in minutes. Today the platform has two substrates:
a distribution of Godot and Web + Three.js. Neither was designed for that
author. Godot is editor-centric: scene files, import caches to prewarm, a 40 MB
Web export, GDScript, and a 2.5 GB executor image to run it headless. Three.js
is a scene-graph library, not an engine: every game reinvents structure, and a
JavaScript game can only be isolated inside an iframe, which costs the play
surface its viewport, gestures, audio and a renderer process. The engine the
agent needs does not exist, so we build it.

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
   in WebAssembly on WebGPU, not in JavaScript on WebGL2.
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
            │  og run --commands "...": bun + @origingame/engine native addon     │
            │  (same Rust core, wgpu→Vulkan on lavapipe or a GPU); same game.js; │
            │  frames, dumps, sweeps, replays as files                            │
            └────────────────────────────────────────────────────────────────────┘
```

### 2.1 Layers

| Layer | Language | Owner | Notes |
|---|---|---|---|
| Engine core | Rust (`crates/og-*`) | engine team | one codebase; targets `wasm32-unknown-unknown` and native (Linux x86_64 for executors, macOS arm64 for developers) |
| Hosts | Rust + generated glue | engine team | `og-host-web` (wasm-bindgen, Worker + OffscreenCanvas), `og-host-node` (napi) |
| TypeScript API | generated `.d.ts` + a thin hand-written ergonomic layer | engine team | generated from one IDL (§3); the ergonomic layer is the three.js-flavoured vocabulary |
| Game | TypeScript | the Maker's agent | components, systems, scenes, tunables, assets |
| Tooling | Rust (`og`) | engine team | one binary: build, run, cook, probe, check, test, doctor |
| Shell | TypeScript (platform `site/`) | platform | the play page and the creator overlay; consumes `@origingame/engine` |

`og check` type-checks game code with tsrs (the Rust port of TypeScript 7),
a pinned binary `og` runs as a child process and never links: from
`OG_TSRS`, the `@tsrs/<platform>` package the engine package brings
through `optionalDependencies`, or `PATH` (`scripts/install-tools.sh`);
where tsrs has no package (Windows, Intel macOS) it falls back to tsgo
through `bun x` and says so. Diagnostics are values: `--json` prints one
`check` event, `{file, line, col, code, message}` per diagnostic, on the
same channel as the frames and the dumps. `og run`, `og test` and `og
build` check before they bundle and stop on a type error (`--no-check`
skips it; `og build --publish` refuses `--no-check`); `og dev` checks each
rebuild beside the bundle and reports it on the `reload` event without
refusing the reload. A checker that crashes or prints nothing readable is
a failure, never a clean check. tsrs is the repository's checker as well
(`packages/engine`'s `typecheck`); tsgo, the compiler it ports, is only
the comparison a wave's end holds it to (`check-conformance`, docs/ci.md)
and the fallback where tsrs has no package.

### 2.2 Threads **(D)**

- **Main thread**: the shell only. Collects input into a SharedArrayBuffer
  ring, owns the AudioContext (unlocked once on the first gesture), the PWA,
  platform UI and the creator overlay. Never runs game code.
- **Engine Worker**: the wasm instance, the renderer on an OffscreenCanvas,
  the game bundle. The tick and the frame run here; a slow frame never stalls
  the shell.
- **Lanes** (phase 3): parallel systems through `wasm-bindgen-rayon` on a
  thread pool of Workers sharing the wasm memory. Requires cross-origin
  isolation (COOP/COEP) on the play page, which the platform already
  declares per publication (`goals/play-surface`). Render recording stays on
  the engine Worker: wgpu's web backend objects are not shareable across
  threads. Without isolation the engine runs single-threaded; the API is the
  same. A lane never changes a result: helpers return in item order, and a
  library that splits across the pool itself (rapier's `parallel` build,
  bitwise identical on any number of threads) runs there only from a grain
  of work for every pool thread, on the engine thread below it.
- **Native host**: the same split (shell = `og`, engine on its own thread,
  lanes via rayon).

### 2.3 Data model **(D)**

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

### 2.4 Time and determinism **(D)**

- Fixed tick (default 60 Hz, per project), accumulator with a cap; render
  interpolates between the last two ticks (`RenderSettings.interpolate`,
  on by default: a display frame blends transforms, the camera, skinned
  joints and world-anchored UI by the accumulator's fraction, a
  frame on a tick boundary and every probe capture draw the tick, so
  reference frames are the ticks'; og-runtime's `interpolate.rs`). Game
  code never reads wall-clock time: `Time` is a resource with `tick`,
  `dt`, `elapsed`.
- Randomness is `Rng` resources seeded from the project seed and the entity
  or system that owns them; the sequence is identical on wasm and native (the
  Web scaffold's `rng.ts` and `rng.rs` test vectors are inherited).
- Snapshot and restore of the whole world (components, resources, rng
  states, pending events) to a blob; `sweep`, hot reload and `feel_compare`
  are built on it, and so are a game's saves (`world.snapshot` and
  `world.restore`, in every build: the blob as text for the platform store,
  restored with the session kept).
- Floating point: simulation uses `f32` with the same operations on both
  hosts; the engine ships its own `sin`/`cos`/`atan2`/`exp` so results do not
  depend on the host's `Math`. Physics (rapier) is built with its
  `enhanced-determinism` feature.
- Large worlds (R12): a world that names `og.WorldOrigin` keeps every
  place near its zero by shifting as a tick starts (og-runtime's
  `origin.rs`, a before-tick hook ahead of cell streaming), when its
  focus (`WorldOrigin.focus`, else the first `og.StreamAnchor`, else
  `og.Camera.position`; a net session refuses the camera) is farther than
  `step` from the zero along an axis of the plane, by the multiple of
  `step` nearest the focus. The shift is world state decided from world
  state, so replays, restores, rollbacks and lockstep peers shift on the
  same tick; the zero is kept in f64 (`WorldOrigin.x/y/z`) and
  `og.OriginShifted` carries the delta for places a game keeps itself.
  What moves is read from the schema, never listed twice: a field marked
  `worldspace world` (a place in the world), `root` (a place in the
  world while its entity has no parent), `hook` (a place under a
  condition its module's state block knows, such as og.Joint's
  `otherAnchor` while `other` is 0) or `none` (not a place), with its axes, in the IDL and the Reflect derive;
  og-idl refuses a `Vector2`/`Vector3` record field whose name reads as a
  place (position, point, target, centre, anchor, origin, pivot, focus,
  eye) without one. `og_ecs::World::shift` moves the marked columns,
  their change shadows with them (a shift is not a change), resources and
  queued events; every `StateBlock` implements `shift` (required, so a new
  block cannot forget it): rapier's bodies and parentless colliders move
  in place, keeping contacts, warm starts and islands, with what slept
  put back to sleep (rapier would wake every contact partner of a moved
  collider), navigation keeps a per-nav frame offset, the og.cells block
  its subtrees, and terrain brushes stay in the logged frame on purpose.
  The render origin (§2.6) is laid on the absolute world's 8192 m grid,
  so a shift never moves it and draws nothing different. 2D worlds shift
  too, on `plane xy`. A power-of-two `step` (the default 2048) keeps every
  shift exact in f32; another step rounds each place once per shift. A
  world that never names it keeps its hash and frames
  (`og-runtime/tests/world_origin.rs`: a scene 100 km out runs bit for
  bit as at the zero, and boxes in contact go through a shift as if
  nothing moved).
- The presentation layer is outside this (the owner, 2026-11-01): what
  is computed for display only, such as a neural network's poses (§2.7,
  motion generation), may differ between devices, precisions and runs.
  It never writes a component the simulation reads, is not in a snapshot
  or `world_hash`, and game code does not branch on it; the simulation
  stays bit-identical on every host.

### 2.5 Scenes and assets **(D)**

- **`.ogt`** is the one text format: a line-based `type`/`version` header, a
  separator, then `key value` lines with nested blocks. Scenes, materials,
  tuning, import settings, project manifest (`project.ogt`, whose `budgets
  { }` hold the asset budgets `og doctor --project` checks) all use it. It
  diffs line by line in git, which matters because the agent's checkpoints
  are git commits.
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
  overrides it for the web export's cook (`og build`, `og run --device`),
  so a game can ship UASTC natively and ETC1S on the web.
- Reusable production assets come through the platform's asset MCP as today;
  the cook consumes what the agent put in `assets/`.

#### Uncooked loads (G3-4) **(D)**

Content a player makes or a generator writes at run time cannot go through
`og cook`, so the runtime reads two formats from bytes, uncooked:
`og.load.imageData` (PNG, JPEG) and `og.load.gltfData` (a `.glb`, or a
`.gltf` whose buffers and images are `data:` URIs). Both answer a pending
handle that turns ready or failed like any load, and the result is drawn as
a cooked texture or model is (`og.Sprite`, material maps, `world.instantiate`,
`og.Model`, `og.load.gltfInfo`).

- **Budgets before decoding.** The source is untrusted, so `LoadBudget`
  (bytes 32 MiB, triangles 500,000, textureSize 4096 and never past 8192,
  the largest every WebGPU device takes, frameMs 2) is checked at the call
  from the headers alone: the file's size, a PNG's IHDR or a JPEG's frame
  header, a glTF's JSON (every triangle primitive of every mesh, each
  embedded image's header). A load over a limit is refused then, naming the
  limit, the value and the fix; nothing is allocated for it.
- **The reader** (og-bundle `runtime::UncookedGltf`, serde_json, no `gltf`
  crate in the wasm) builds the same `CookedModel` the cook's blob decodes
  to, and the model joins the renderer through the insert path cooked
  models take. It reads what the cook reads that needs no cook: the
  metallic-roughness materials and their maps, `KHR_materials_unlit` and
  `_emissive_strength`, nodes and scenes, vertex colours and tangents. A
  colour texture magnified nearest is pixel art. What only the cook makes
  is absent: levels of detail, cooked raycast BVHs (a raycast builds one
  when first needed), meshopt-coded streams, compressed textures. Skins and
  animations are skipped and counted. What would draw wrong is refused by
  name: Draco and meshopt compression, Basis, WebP and AVIF textures,
  sparse accessors, a buffer or image in another file, any other required
  extension.
- **Off the engine's thread.** Images decode in the browser on the web
  (`createImageBitmap` and an `OffscreenCanvas` in the engine's Worker,
  behind a host hook; no image decoder is in the wasm) and on a worker
  thread natively (the `image` crate). A glTF's primitives are converted at
  the tick boundaries, one at a time within `frameMs` on the web (at least
  one a frame), all at once natively.
- **Completion is host input.** On the web the tick a load lands on depends
  on the browser, so each load is put in place at a tick boundary and
  journaled as `{ request, tick, ok }` (`og.load.completions`), as G3-6
  journals the platform's answers. `og replay --record` keeps the journal
  in its trace; `--compare` and `og.load.replayCompletions` hold each load
  until its recorded tick and put it in place there, waiting for it, and a
  recorded failure fails. Natively a load is in place at the boundary after
  its call, the same every run. An asset's status is not world state: it is
  in no snapshot or hash. In a lockstep or rollback room, content made by
  one player is not on the others' machines, so a completion stays local
  to each session. A game sends the bytes to the other peers with
  `og.platform.roomSend` (at most 64 KiB a message; room messages are
  session events, not hashed), each peer loads them, and the shared
  simulation uses the content only from a tick the peers agree on through
  their inputs (`og.net.submit`), never from `og.load.status`.
- **Release.** `og.load.release` frees an uncooked asset. A pending load is
  dropped and never journaled. A ready one's textures and meshes are emptied
  under their handles, as a stream group's stand-ins are, so whatever still
  names them draws nothing and nothing breaks. Their GPU memory and the
  scene's CPU copies go, with the model's draw parts and cached BVHs. The
  handle answers failed. Physics shapes built from its geometries are world
  state and stay. `dump_memory` (§3.3) shows both sides.

#### Prefabs (G2-1) **(D)**

A prefab is a reusable entity tree authored once and placed many times.
The design keeps the scene path as it is (one text grammar, one cooked
scene blob, one spawn) and adds deltas on top of it: an instance stores
only what differs from its prefab, so a prefab edit reaches every
instance on the next cook and an instance's edits survive it.

**The asset.** `assets/**/*.ogt` of `type prefab`, version 1: exactly one
top-level `entity "<root>" { ... }` in the scene grammar (components,
`model`, child `entity` blocks), and nothing else (no `resource`,
`path` or `stream`: those belong to the scene that places it). Entities
inside a prefab are addressed by their **node path**, the names from the
root down joined by `/` (`""` is the root, `lid/hinge` a grandchild);
names are unique among siblings, which the cook checks.

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

The instance's own component blocks and its `override "<node>"` blocks
list only fields that differ; a component the prefab node lacks is added.
`remove "<node>"` drops a node and its subtree; `remove "<node>" Health`
drops one component. Instances nest: a prefab may contain instances of
others, to any depth; a cycle (a prefab reaching itself through its
instances or its base) is refused naming the chain.

**Variants.** A prefab whose header body starts with `base
"prefabs/goblin.ogt"` is a variant: its one `entity` block is an instance
of the base, in exactly the instance grammar above (overrides, `remove`,
added children). Variants chain (a variant of a variant); the root keeps
the variant's own entity name.

**Cooking.** The cook flattens. A scene (or prefab) is expanded
depth-first in text order: an instance becomes its prefab's flattened
tree at its place, then the variant chain's deltas from the base
outwards, then the instance's own, each in text order. The scene blob
the runtime reads is unchanged in shape (entities, parents, component
tables), so spawning, hooks (§2.3, G2-2) and the world hash treat a
flattened prefab exactly as the same entities written out by hand. Two
things are added beside it:

- a `prefabs` table: per entity, the prefab asset id and node path it
  came from, and the index of its instance root (or none for an authored
  entity), for the runtime's link (below) and the tools;
- the stamp: a scene's (and a prefab's) cook stamp includes the source
  hash of every prefab it reaches, transitively, so editing a prefab
  re-cooks every scene and prefab that places it. Prefabs cook to a blob
  of their own too (the same scene blob, flattened), so
  `og.load.prefab` loads one for `world.instantiate` at run time.

Overrides are kept on re-cook by construction: they are text in the
instance, applied over whatever the prefab says now. A field the
instance does not override follows the prefab; one it does keeps its
value. An **orphaned** override or `remove` (its node no longer exists:
the prefab renamed or deleted it), or an overridden field the prefab's
component no longer has, stays in the text so a rename back, or a fix,
restores it. A dev cook warns in its notes; `og prefab check` exits
non-zero while any exists; a publish cook refuses it, naming the
instance, the node and the field: a game never ships an override that
silently does nothing (the owner, 2026-11-02). Everything else the cook
refuses as it refuses a scene today (unknown asset, bad values, a
cycle).

**Names.** An instance's root takes the instance entity's name; each
descendant is named `<root name>/<node path>` (`crate1/lid/hinge`), so a
placed tree is addressable by the probe, `world.find` and the command
channel, and match the probe's path ids. A generated name that collides
with a hand-written entity's or another instance's fails the cook,
naming both. An unnamed instance root leaves its tree unnamed.

**At run time.** Each entity spawned from a prefab carries one engine
component, `og.PrefabLink { prefab: Asset, node: u32, root: Entity }`,
written by the spawn from the `prefabs` table: `prefab` is the prefab's
asset handle (interned, never a string per entity) and `node` the index
of the node in that prefab's flattened table, which names its path. It
is ordinary world state: snapshots, restores and the hash carry it, so a
game that places prefabs hashes differently from the same entities
written by hand (accepted, the owner, 2026-11-02), and stably across a
re-cook that changes nothing it spawns. A game without prefabs has no
such entity, so its archetypes, saves and hash are unchanged, and older
saves need no migration (no saved layout changes). The API: `og.load.prefab(path)` (an `Asset`), the existing
`world.instantiate(asset, SpawnDesc)` (a prefab is instantiated like a
scene, under a root from the desc, and that root is the instance root),
and `world.prefabOf(entity)`, the link as `Option<PrefabLink>`. og-ecs
needs nothing new: the link is a reflected component, the spawn order is
the flattened order, and add hooks fire per entity as they spawn.

**Write-back and apply-back (dev builds).** `save_scene` keeps
instances as instances: for an entity with a link whose root is a scene
instance, a tuned field is written as an override line in that
instance's block (`override "<node>"`, or the instance's own block for
the root), and only when it differs from the prefab's cooked value; a
field tuned back to the prefab's value has its override line removed.
`apply_prefab:<entity>[:<Component>[.<field>]]` (a command-channel
action; no RPC member, so `hello`'s members are unchanged) is the other
direction: the instance's overrides on
that entity's node (all of them, a component's, or one field) are
written into the prefab's `.ogt` (through the same line-preserving
rewrite, at the node's block) and removed from the instance's text, so
the scene cooks to the same world and every other instance of the
prefab takes the new value. On a variant, apply-back writes into the
variant's own text, and `:base` writes through to the base. A value
that differs only on this instance and was never an override (a run
time change through `set`) is applied the same way. `revert_prefab:
<entity>[:<Component>[.<field>]]` removes overrides and sets the live
values back to the prefab's.

**Tools.**

- `og prefab check [project]`: cycles, orphaned overrides and removes,
  overridden fields the prefab no longer has, sibling name clashes.
- `og prefab show <prefab> [--scene <id> --instance <name>]`: the
  flattened tree, each field with where its value comes from (base,
  variant, instance override).
- `og prefab extract <scene> <entity> <prefab path>`: an authored entity
  subtree becomes a new prefab and the scene's block becomes an instance
  of it with no overrides (the scene cooks to the same world).
- Probe: `dump_entity` gains `prefab` (asset, node, root, and the fields
  this instance overrides); `apply_prefab`, `revert_prefab` as above;
  `describe` lists instances by prefab. Strictly additive to protocol 5:
  the actions are new, the dump field optional, and nothing existing
  changes shape (origin-agent's probe.ts follows when this lands).

**Determinism.** Flattening is a pure function of the texts (depth-first,
text order, deltas in a fixed order), so both hosts spawn the same
entities in the same order with the same values, and a replay or a save
does not depend on whether a tree was written by hand or placed from a
prefab, apart from the `og.PrefabLink` components it carries.

#### Sub-scenes and world cells (G3-3) **(D)**

Streaming groups (§2.6's assets, IDL `stream`) move what a frame draws
and plays; nothing a tick reads depends on them. World cells move
entities, which a tick does read, so their rule is the opposite: what is
loaded at a tick is a function of world state alone, never of I/O
timing.

**Sub-scenes.** In an entity block, `scene "<id>"` places another scene
there. Unlike a prefab it is not flattened: it stays its own blob, named
in the placing scene's optional `subscenes` table, so editing it re-cooks
it alone, and a sub-scene that reaches the placing scene again is refused
naming the chain. Without a cell, the runtime spawns it under the entity
as it spawns the scene (`world.instantiate` under a root).

**Cells.** An entity with a sub-scene and `og.StreamCell { scene, size,
load, unload, pin, loaded }` is a cell. As each tick starts (the probe's
before-tick hook, so a replay, a rollback's re-simulation and a restored
world all run it, before any system), the engine decides from the
world's state whether each cell is loaded: an `og.StreamAnchor { reach }`
entity within `load` of the box (the cell's world position ± half
`size`, turned with it), kept until every anchor is past `unload`
(hysteresis), or `pin`. Without anchors the `og.Camera` resource's
position anchors, a single-player convenience refused while
`og.Lockstep.enabled` (each peer's camera is its own). A load spawns the
sub-scene: its top-level entities are roots at the cell's pose after
their own (physics bodies are roots), every spawned entity carries
`og.CellMember { cell }`; an unload despawns every member and its
descendants, wherever they moved (a game keeps one past its cell by
removing the member). Each change writes `loaded` and sends
`og.CellEvent { cell, loaded, entities }`, delivered in the same tick,
and G2-2's add hooks and `added` filters see the entities arrive.

**Order.** Cells are visited by entity (slot, then generation); a
sub-scene spawns in its blob's order (text order, depth first); an
unload despawns children first, by entity. The same world therefore
allocates the same entity ids on both hosts, in a replay and after a
restore (pinned in og-runtime's `tests/cells.rs`).

**Snapshots and saves.** `loaded` is a component field and a loaded
cell's entities are ordinary entities, so a snapshot, a save, the hash
and a rollback carry exactly which cells are loaded and what is in them;
an unloaded cell's entities are not in the world and come back from its
sub-scene, as authored, when it loads again. `StreamCell.scene` is an
asset handle, saved as its number like every handle: a session loads the
scenes that place the cells (GameDesc.scene does) before restoring. The
four records are lazy (og-runtime's `lazy`): a world without cells keeps its
type table, saves and hash.

**Persisted cells and the spawn budget.** A cell with `persist` keeps what
happened in it: as it unloads, its members and their descendants (parents
first) are kept as rows (saved id, name, every component with its layout
hash) in the `og.cells` state block, added with the cell's records, so a
snapshot, a save, the hash and a rollback carry them. As it loads again the
rows spawn instead of the sub-scene, every entity field that named a kept
row pointed at that row's new entity, and the rows are dropped (the
entities are in the world again); rows kept by a build whose components
have another layout are refused with a warning and the cell loads as
authored. `budget` caps the entities a load spawns per tick; the load in
progress (its source and the ids spawned so far) is in the same block, so
a restore taken mid-load goes on from there. `loaded` turns true and
`og.CellEvent` is sent the tick the last entity spawns; a cell no longer
wanted before then despawns what it spawned and sends nothing.

**A cell that is not ready.** A sub-scene's blob is in the boot bundle
unless the sub-scene declares a cell group, `stream "<name>" { cell;
asset ... }` (at most one, only in a scene some scene places, and with no
distance or volume): then the cook packs the blob into that group with
the assets it names, its output names the group, and the runtime's
handle for it waits (`Payload::SceneWaiting`) until the pack has landed,
and again after the group unloads. When the pack is read is presentation:
at every tick boundary a cell group is wanted while one of its cells is
loaded, loading or pinned, or an anchor 2 s further along at its
`StreamAnchor.maxSpeed` would be within the cell's load distance (plus
its reach), so at the declared speed it has landed before the cell
loads. What a tick does is not: a tick that would load a cell (from its
sub-scene, not from kept rows) whose blob has not landed is held by a
gate chained before og.net's, never skipped or reordered. The gate first
asks for the group, first in admission whatever its priority, and reads
it there; natively a read completes at once, so a native run or replay
never holds. A host that answers the read `Pending` (the web's fetch)
holds the tick: frames go on drawing, `dump_stream`'s `hold` names the
tick, the cell, the group and its state, and once the tick goes on the
engine's log records the hold and how long it lasted (`holds`: their
count, total and longest). After a rollback's rewind the gate asks again
for the re-simulated tick. The world a held tick goes on with is the one
a run that never waited has at that tick, apart from og.StreamEvent,
which is world state: the ticks that deliver a group's events hash them,
streamed or not. Its models, textures and sounds draw their stand-ins
until the group is resident, as any grouped asset does, and physics only
ever reads colliders from components and proxies.

**Baked colliders.** A cell's ground is usually a triangle-mesh collider
over a model's geometry, and building that shape (parry's mesh and its
BVH) in the step that first meets it is the cell's worst tick (131,072
triangles: 49 ms). A model cooked with `model { collider true }` carries
each primitive's shape baked over LOD 0 exactly as the runtime decodes the
blob (the proxy's level for a streamed model, the geometry physics
registers), in its `colliders` section; the runtime gives it to physics as
the model loads (decoded there, 18 ms for that mesh), and a SHAPE_TRIMESH
collider at scale 1 naming the geometry takes it while the mesh is
bit-for-bit the one baked, so the shape is the one a build would make and
nothing a tick computes changes (that step: 9 ms). A scaled collider, a
convex hull and the 2D profile build as before. It is opt-in: about 90
bytes a triangle.

**Texture mips.** A texture cooked with `texture { stream true }` (or a
model's, `model { stream_textures true }`) ships twice from one encode:
`<id>`, its mip tail (the whole chain's levels from the first at most
256 px on its longest side, byte for byte), in the boot bundle, and
`<id>#mips`, the whole chain, fetched on demand (the web host never
preloads it; a publish build caches it lazily, as sound chunks). A texture
already within 256 px, pixel art and one in a streaming group (its pack
holds the whole chain) cook whole. Before each frame
(og-runtime's `stream/mips.rs`), each streamed texture's footprint is the
most pixels across any instance drawing it covers (its mesh's bounding
sphere, measured as `og_render::lod` measures); one whose footprint passes
its tail's longest side is wanted whole, and stays wanted 120 frames
after it was last, so a camera turning away and back does not read it
again. The wanted ones are admitted by footprint over tail (then id, the
rank kept while they stay wanted) within a mip budget of 128 MiB, apart
from the groups'; they are read through the streaming fetcher after the
sound chunks and the packs, transcoded by the host and swapped in under
the same handle, and one no longer admitted goes back to its tail.
`texture_size` reports the whole chain's size throughout, so nothing
sized from the texture moves. It is presentation only: no world state,
nothing a tick reads. The probe's `mip_stream:resident` reads and shows
every streamed texture whole before each frame (natively at once, so a
capture is the whole texture's, as without streaming);
`mip_stream:stream[:<MiB>]` streams again within that budget, and
`dump_stream`'s `mips` lists the mode, the budget, what is used and each
texture's state and footprint (`og-runtime/tests/stream_mips.rs`).

### 2.6 Renderer **(D)**

wgpu with WGSL shaders cooked per backend (naga). On the web the renderer
is WebGPU only (the owner, 2026-11-01, §8): compute is available on every
supported device, and a feature is designed for the full tier rather than
cut to a reduced one. The WebGL2 tier (wgpu's GL backend, CPU culling and
particles, a CPU-built forward tile table, a joint palette in a texture)
was removed with that decision; the CPU culling and CPU particle paths
remain as options on every device (`Culling::Cpu`, small frames under
`Culling::Auto`, `ParticleSimulation::Cpu`). The engine is GPU only on
every host (the owner, 2026-11-02): a player's device without a GPU is
not supported, and no feature adds a CPU fallback for one. Developer
tooling keeps running without a GPU (the owner, 2026-11-02): the logic
tests in orbs and CI, and the probe's CPU description of a frame.

- Order-independent transparency for the main camera's blended surfaces
  (`oit.rs`, `RenderOptions.transparency`): weighted blended (McGuire and
  Bavoil 2013) with each pixel's nearest layer exact (hybrid transparency,
  Maule et al. 2013; marked by an alpha-only depth pass into the same
  arena) by default; per-pixel lists as the high tier, one packed
  atomic arena at group 0 binding 14 with a fixed node budget a pixel,
  sized against the device's own limits, every fragment also weighted and
  a pixel past its budget resolved whole as weighted and counted (stats,
  `dump_render`). Resolved per MSAA sample before TAA, whose history
  weight the revealage lowers (a reactive mask). Additive effects, 2D
  sprites, text, the UI and secondary cameras keep their order.
- Forward+ (clustered), with a depth-normal prepass when an effect reads
  depth or normals (on every frame it cost the 20,000-sphere field two
  thirds of its frame and bought nothing: `tests/evidence/P4/perf/prepass.md`);
  PBR metallic-roughness
  (glTF-compatible), normal, emissive, occlusion, and three.js's physical
  lobes (clearcoat, sheen, transmission, iridescence, anisotropy).
- Cascaded shadow maps for one directional light (opt-in PCSS soft
  shadows and blending across cascade splits); cube faces or two
  dual-paraboloid hemispheres for up to 8 point/spot lights, the brightest
  at the camera first (cube by default, paraboloid as the cheaper choice;
  `RenderOptions.pointShadows`). A local light's layer holds the midpoint
  of its casters' near and far sides (Woo's midpoint shadow map, two
  rasters per face, the second mirrored: `shadow_mid.rs`), so neither a
  closed receiver nor a zero-thickness one compares against itself. The
  cascades still store far sides and compare a closed receiver past its
  own depth (a follow-up: the same midpoint for them).
- IBL from an environment map prefiltered at cook time, as three.js's
  scene.environment and background: a blurred background read from the
  prefiltered levels (`backgroundBlurriness`), a grounded dome
  (GroundedSkybox, `groundHeight`), and a blend towards a second map
  (`blendTo`, `blendWeight`, drawn in 128 steps and driven from the tick)
  that mixes every prefiltered level and the harmonics; uncooked `.hdr`
  (and natively `.exr`, `RenderCapabilities.exrDecode`) loaded at run time
  by `og.load.environmentData`, decoded off the engine thread or in
  budgeted rows, and GGX-prefiltered on the GPU by the IBL passes (the
  cook's CPU `bake` stays for cooked maps and tools); sky model with
  exposure; height fog, lit as a medium when volumetric; baked lightmaps
  and light volumes (below).
- Post: bloom, tonemap (ACES, AgX or Khronos PBR Neutral), colour grading LUT (a photo
  editor's colour adjustments baked into it on the CPU), lens distortion, zoom, spin and directional screen blurs, SSAO (GTAO on
  WebGPU), antialiasing: TAA by default (optional Catmull-Rom history
  and sharpening), 4× MSAA or FXAA as choices, FXAA where the HDR target
  cannot be multisampled. TAA reprojects along the prepass's motion
  vectors where something moved besides the camera (instances with
  `previous_transforms`, skinned meshes with `previous_joint_palettes`,
  both filled by og-runtime from the frame drawn before, interpolated or
  not; under GPU culling instances move with the camera only), and below
  render scale 1 it is temporal upscaling (TAAU, R2): the scene is drawn
  at the render scale with a Halton jitter over 8 phases per output pixel
  a scene pixel covers (up to 32), TAA writes an output-sized history
  from the sample nearest each output pixel, weighted by its distance,
  and every pass after it runs at the output's size; a change of scale
  below 1 keeps the history (dynamic resolution without a reset). The
  resolve is a quality field (`RenderSettings.upscaler`): TAAU, or AMD
  FidelityFX Super Resolution 3.1's upscaler ported to WGSL in fp32
  without subgroups (MIT, `third_party/fidelityfx/`), which takes the
  `taa` pass's place with its eight compute passes (depth-reconstructed
  disocclusion, thin-feature locks, luma instability, Lanczos
  accumulation, the OIT coverage as its reactive mask) and its RCAS in
  `taa-sharpen`'s; its state is sized to the output, so it keeps its
  history across scale changes too. `docs/upscalers.md` compares the two
  and gives the presets' choice.
- What a frame may cost is data the renderer reads every frame:
  RenderSettings' quality fields (rect-light shadows, the number of
  shadowed lights, the shadow maps' scale, volumetric fog, soft particles,
  SSR, AO and its resolution, the SSR and volumetric fog step scales,
  MSAA samples, the upscaler), og-render's `RenderQuality`. A graphics preset
  (`GraphicsQuality`: low, medium, high) is a row of them that
  `og.render.applyQuality` writes and `RenderSettings.quality` records;
  no preset state lies behind the fields, so the probe reads what is
  drawn. Medium is the defaults, the full frame. High refines it: shadow
  maps at twice their side, AO at full resolution whatever PostEffects
  asks, twice the SSR steps and the volumetric fog's slices. The device
  caps two fields and says so in RenderCapabilities: `maxShadowedLights`
  and `maxMsaaSamples` (1 where the renderer resolved TAA, FXAA or none);
  the fields keep the row's value.
- Stylisation, as data a look composes (`og.Stylise`, `og.MaterialStyle`,
  a `type look` asset): a logical pixel grid scaled up by whole numbers
  with the camera and sprites snapped to it, posterise and palette
  quantisation through ordered or blue-noise dither, halftone,
  cross-hatching, ink lines, painterly (Kuwahara) smoothing with pigment
  pooling, bleed and a canvas, flat depth bands, retro-3D vertex snapping,
  affine texturing and coarse depth, and scene-wide material defaults
  (toon, rim, outlines thinning with distance, tinted shadows, flat
  facets); every primitive off at zero and then free, on both tiers.
- Skinning on the GPU (storage buffers; vertex-texture fallback on GL).
- Instancing by default: draws are batched per material and mesh; the
  TypeScript API has no per-object draw calls to misuse.
- Particles: GPU simulation on WebGPU, CPU on GL; one emitter asset format.
  Collision against the depth buffer in compute, against the scene's
  colliders (ground, boxes, spheres, terrain height fields) on the CPU.
- Terrain: clipmap heightfield with splat materials, its field read from a
  heightmap or made by the cook (`generate`: fractal noise, an island or
  ridge mask, hydraulic and thermal erosion, terraces, a sea with its
  shore, rivers along the routed flow, biome rules writing the control
  map), with IEEE-exact arithmetic and the generator's own PCG32 so every
  host makes the same field; both routes write the same tile pyramid, and
  `TerrainInfo.seaLevel` places the water. Detail meshes scattered over
  it sway in `og.Wind` (a lean downwind with gusts that sweep across the
  field, drawn from the tick and never read by the simulation). Decals:
  deferred-style projection in the forward pass.
- Text: MSDF for UI and world text, kerned from the font's GPOS pairs;
  CJK coverage through the platform's subset fonts, and characters outside
  a cooked charset rendered at run time from the font's outlines (`font {
  runtime true }`). A pixel font (`font { pixel 12 }`, its pixels to
  the em) is cooked as its cells on that grid, sampled at the cells'
  centres and traced into square contours before the field, its advances
  and kerning in whole cells, and og-ui sets it at whole multiples of the
  grid only, so each font pixel is whole CSS pixels on the snapped pen
  (run-time glyphs take the same cells). Bidi (UAX #9) and shaping (Arabic, Indic) through
  rustybuzz, behind og-ui's `shaping` feature. World text faces the camera
  as a billboard does when asked (`TextDesc.facing`), turned as one panel
  about its aligned point and sorted among the transparent meshes.
- Capture: a readback path from the render graph, used by the probe's
  `capture` on both hosts; on the web it reads the OffscreenCanvas target, so
  no `preserveDrawingBuffer`.

**Meshlets** (the realism wave's R1, after three.js#35191's
`MeshletDrawingPipeline` and `VisibilityBufferDrawingPipeline`;
`meshlet.rs`). The cook cuts every level of every model primitive into
clusters of at most 64 triangles (meshoptimizer's `buildMeshlets`, up to
192 distinct vertices, so a faceted mesh still fills them) and reorders
the level's draw indices so each cluster is a contiguous range of them.
The model blob keeps that stream in `cluster_indices`, separate from
`indices`: the simulation mesh, its vertex-fetch order and its raycast
BVH retain the pre-meshlet order. Reordering a collision mesh changes
ray hits and contacts through floating-point arithmetic; a rendering
optimization must not change the simulation. Both streams use the same
vertices and LOD ranges. Each cluster keeps its bounding sphere (widened
to its farthest vertex) and
its normal cone (the model blob's `clusters` table, each `lods` row
naming its range). The levels are the ones per-instance LOD already uses
(`lod.rs`): simplified levels with their measured error, not a cluster
DAG (the owner, 2026-11-02: three.js#35191's model, as the roadmap's R1
names it). Its limit: one level is chosen for a whole instance, from the
error projected at its nearest point, so one huge mesh (a building, a
terrain chunk, a long wall) draws its nearest part's detail everywhere:
the far end of a building seen along its length keeps the full mesh, and
only the culling of its clusters saves work there. Large assets are
best authored in parts. A cluster DAG (Nanite's: clusters of each level
simplified in groups, each choosing its own detail) lifts the limit
without another blob format: every cluster row already carries its own
error, its parents' error, the sphere both are projected from and its
group, and a cluster is drawn when its error projects under a pixel and
its parents' does not (`meshlet::selected`); the cook's per-level chain
writes those fields so that exactly the level `lod::select` picks is
drawn. Per frame (`meshlet_gpu.rs`, `meshlet_cull.wgsl`), after GPU
culling (`cull_gpu.rs`) has culled and compacted a clustered batch's
instances, one compute pass runs a workgroup per instance: it tests the
instance's sphere against the frustum and the depth pyramid, picks its
level by screen-space error (`lod::select` without hysteresis, with the
settings' `lodBias`) unless its caller picked it
(`RenderScene::instance_levels`; og-runtime picks one level per model, by
its parts' largest error, its own `lodBias` and hysteresis, as for the
regular path's parts, so both paths draw the same picture and every view
and shadow pass draws that level), then tests each of the level's clusters on its
own: its sphere against the frustum, its normal cone against the eye
(not for two-sided materials, transforms that mirror or scale unevenly,
or orthographic cameras), and its sphere
against a depth pyramid, in two phases of its own (`meshlet_cull.wgsl`
`vis` keeps one bit per cluster slot across frames): the early phase
takes the clusters visible last frame that pass the frustum and cone,
and draws those of occluding materials into GPU culling's occluder depth
(already holding last frame's visible occluders, drawn at this frame's
camera); the pyramid is built again from it; the late phase tests every
cluster against that pyramid, records the visible ones for the next
frame, and adds those the early phase did not draw. The pyramid holds
only depth drawn at this frame's camera, so occlusion never removes a
visible pixel, whatever last frame left. A frame without GPU culling's
pyramid (the first, or no occluding material) draws what the frustum and
cone keep in one phase and records it. The
visible clusters are appended as work items (instance, cluster), and
each batch is drawn once, by `draw_indirect`, 64 × 3 vertices per
visible cluster, by vertex pulling (`meshlet.wgsl`: no index or vertex
buffer is bound; a cluster's unused triangles collapse), in the prepass
and the lit pass. The prepass reads each instance's previous transform
at its sorted slot: under TAA with object motion GPU culling uploads the
sorted instances' previous transforms beside the current ones
(`cull_gpu.rs` `previous`) and its camera pass compacts them with its
survivors, so the regular prepass, the meshlet path and the visibility
buffer's setup all give TAA and object motion blur each object's own
motion. `debugView lod` tints each cluster by its
level. The path takes the camera pass's opaque batches of clustered
meshes under GPU culling (`Culling.auto` turns GPU culling on when such
a batch exists); what it cannot take keeps the per-instance path:
skinned, morphed, lightmapped or compact meshes, custom shaders,
transparent and physical materials, texture arrays, runtime geometry
without clusters, object-space stylise, and `Culling.cpu`
(`meshlet_gpu::takes`, `RenderScene.meshlets` turns it off). The shadow
maps go through the same dispatch: each cascade and each local light's
face is a view of its own, whose rows test the clustered casters (on
every layer) against that pass's planes alone, with no normal cone (a
front-sided caster draws its back faces) and no pyramid (it is the
camera's), at the level the camera picked, so a caster's shadow has the
shape it is drawn with; they are drawn by vertex pulling in the shadow
pipeline's state (`meshlet_shadow.wgsl`; alpha-tested and dissolving
casters with their material). A view whose work would pass the
device's storage binding limit keeps the per-instance casters. The
cascades' setup, splits and maps are unchanged. The planar
reflections, views and reflection probe faces are views of the same
dispatch too, each with its planes, its eye and its pixels per unit (half
its height times the length of its view-projection's clip y row, which
holds for perspective, orthographic, mirrored and oblique projections),
so each picks its own levels and culls by the cone from its own eye; only
the camera tests the pyramid. A view or a probe face with a depth prepass
(a depth texture, soft particles) keeps the per-instance path, as does
any view past the storage binding limit.

The visibility buffer (`meshlet_gpu/vis.rs`, `meshlet_raster.wgsl`)
takes the camera's small clusters: of an occluder batch (opaque, with no
alpha cut, dissolve or material shader), under a perspective camera
(`RenderScene::visibility_buffer`, on by default), a cluster wholly past the near plane whose sphere covers at
most 24 pixels across (`meshlet::soft`) goes to a soft list instead of
its batch's indirect draw. One workgroup per soft cluster and one thread
per triangle rasterise it in compute, in two passes because WebGPU has no
64-bit atomics: an atomic max of the depth word, then an atomic max of the
id word (the cluster's slot in the list and the triangle) where the depth
is the pixel's. Vertices snap to 1/256 of a pixel inside the triangle's
box, and the top-left rule makes triangles that share an edge cover each
pixel centre once (`meshlet::covers`). A setup pass, one thread per
pixel, rebuilds the seen triangle's attributes with perspective-correct
barycentrics (`meshlet::barycentrics`) into a record. Then one
full-screen resolve per soft batch, in the camera's lit pass and its
prepass, shades that batch's pixels with the same `lit_fragment` and
`prepass_targets` as the hardware path, writing the rasteriser's depth,
so the larger clusters, still drawn by the hardware, meet the resolved
pixels by the depth test. A full-screen pass's 2x2 neighbours are other
triangles' pixels, so its screen derivatives mean nothing (a texture would
sample its coarsest mip): the setup also records each pixel's uv and
world-position differences to the next pixel right and down on its
triangle's plane, and the resolve's pipelines set `pbr_fs.wgsl`'s
`ANALYTIC`, which samples the material's maps with those gradients
(`textureSampleGrad`) and takes them as the position's derivatives.
Retro 3D's vertex path is repeated in the rasteriser: each corner snaps
to the frame's grid and its depth to its steps before it is rasterised
(`meshlet::retro_clip`, pbr.wgsl's `retro_clip`), and with affine
texturing the setup records the screen-linear uv (`meshlet::screen_weights`:
the hardware's perspective-correct `uv w` over `w`). Under the material
style's surface or affine texturing each record carries three more rows,
the local position's gradients and the lit uv's, so the surface pattern's
fade, coarse albedo and height bump (the pattern again at the next pixel
right and down) are the hardware's. three.js decides per triangle (8 pixels); per
cluster keeps each hardware cluster in its batch's one indirect draw. The
resolve's fragment stage adds one storage buffer (each pixel's record
slot, then the records) and one uniform to the lit stage's, and no
texture, since the lit fragment stage already samples 15 of WebGPU's 16.
Reflections, views, probe faces and shadows keep the hardware path, as do
orthographic cameras.

Foliage (the vegetation's trees) sways on the meshlet path. A primitive's
glTF `_OG_PIVOT` and `_OG_ANCHOR` (branch pivot and stiffness, leaf anchor
and flutter, VEC4 floats) and `extras.ogFoliage.leaves` cook into the
model's optional `foliage_rows` and `foliage` sections: welding keeps
vertices of other pivots apart, a simplified level locks every vertex on
a seam between two pivots or anchors (and, simplified by position, every
position two branches share), and leaf cards get no simplified levels.
`RenderScene::set_mesh_foliage` puts a mesh's foliage
(`meshlet::MeshFoliage`) after the vertex pool, each cluster's first row
naming its region; every pull (the lit pass, the prepass at the last drawn
frame's wind and sway, the occluders' depth, the shadow casters and the
compute rasteriser) moves the vertex by `foliage_offset` at
`Instance::sway` (the record's layer word for runs without array maps).
The cull widens the mesh's and its clusters' spheres by its deformation
bound (`FoliageBounds::deform_margin`, the larger of this frame's wind and
the last drawn's, times the sway when over 1) and takes no cone from them.

The CPU statement of every test is `meshlet.rs`, whose unit tests check it.

Runtime geometry and debug draw (depth S1). Games can make geometry from
attribute arrays (`og.render.bufferGeometry`, three.js BufferGeometry's
attributes, with normals computed when they are missing) and from three.js's
primitives: cylinder, cone, capsule, torus, ring, circle, and a tube along a
centripetal Catmull-Rom curve or along an og.path (its rings evenly spaced
by length, as three.js's). A dynamic geometry is rewritten in place
each tick (`updateGeometry`). A made geometry is also a physics mesh
under its handle, so an og.Collider can use it as its `mesh`. Each geometry
has a BVH, built when it is made or updated; a model's is cooked into its
blob (`model { raycast false }` leaves it out, and the first raycast that
reaches the model builds it);
`og.render.raycast` uses it to test what is drawn rather than the
colliders. Inside a system, the raycast reads the meshes as the probe
recorded them at the tick's start.

Procedural modelling (`crates/og-geom`) makes three.js's modelling
geometries: shape (earcut, a port of three.js's own), extrude (bevel, steps,
along an og.path with Frenet frames), lathe, polyhedron with the four
regular solids, a section swept along a path with scale and twist keyed
over its length, merge of placed parts (mirrored, arrayed), and CSG
(union, subtract, intersect). The crate depends on glam, `robust`
(Shewchuk's predicates) and `spade` (a constrained Delaunay triangulation
of each cut face), never on og-render. Its mesh converts into a MeshData
in og-runtime's `geometry.rs`, and from there it goes the same way as
`bufferGeometry`: a BVH, `physics`, `dynamic`, `geometryInfo`. The
counts, vertex orders and uvs are three.js's. Every result is the same on
every host. The CSG decides each plane side and edge crossing with exact
orient3d and orient2d. Pieces are classified by a winding number, and
coincident faces keep one copy or none, as the operation says. The
generators take sine and cosine from series evaluated on integer segment
indices, never from libm. `og.Shape` and `og.Path2` are TypeScript
(three.js's Shape and Path); they flatten to points before they cross the
ABI.

A model can be written as text from the same parts: an `.ogt` of `type
model` (og-cook's `cook/model_text.rs`) names each part's geometry by its
og.render call and desc fields, places it as mergeGeometries does, gives
it a material and hangs it from a named pivot. The cook writes it as a
glTF binary (the writer `.vox` uses) before the model path reads it, so
it becomes the one model blob a glTF does (an FBX or OBJ goes the same
way, read by ufbx into a glTF binary in og-cook's `cook/fbx.rs`): meshopt, the LOD chain with
measured errors, the raycast BVH, the node graph. A part placed several
times is one mesh under several nodes, drawn instanced; a mirrored
placement is baked. The cook refuses it past the project's triangle and
vertex budgets. og-geom's `path2.rs` flattens the text's og.Shape calls
with the TypeScript runtime's formulas and series, so a shape written in
either place gives the same points, bit for bit.

Lines and points are one pass (`lines.rs`, `points.rs`,
`shaders/lines.wgsl`). Each segment or point is an instance of a quad in
screen space with a feathered edge, so widths in pixels or in world units
and dashes behave the same on every backend; WebGPU has no line width at
all. Material `wireframe` draws its instances'
unique triangle edges through the same pass, and the edges are computed
once per mesh revision. The pass draws in two places:

- **Scene**: in the main pass after the transparent meshes and particles,
  depth tested or not.
- **Overlay**: over the finished frame, after every post effect and before
  the UI. Debug draw (`og.debug`), the debug views and screen lines draw
  here, so they read the same in every capture.

The debug views are a `RenderOptions.debugView` enum (none, physics, nav,
bounds, lod, and the lit image's views below) that later streams add
variants to; `og.debug.setView` and the command channel's `debug:` action
switch them. The physics view is rapier's debug render of the last step:
collider shapes, joints and contacts. The nav view draws the grids,
navmesh polygons and agents' paths, and the bounds view draws mesh
bounds. The lit image's views replace the lit colour with one term of
it, written by the lit shaders themselves (`RenderScene::debug`, the
frame uniform's `light_color.w`): `cascades` (the directional light's
cascade per pixel), `ao` (the ambient occlusion term), `albedo`,
`iblDiffuse`, `iblSpecular` and `direct`; `aoDepth` and `aoRaw` show the
ao view with the AO pass's input depth or its term before the blur
(`FxPost.cam.w`). They are how a port from three.js compares its lighting
term by term. Debug draw is rendering: it is not world state, it never
feeds the tick, and the `debug` module is left out of publish builds,
whose frames draw none of the lit image's views.

Shading diagnostics never silently substitute the beauty image. An
unsupported 3D surface/view pair draws an eight-pixel magenta/black checker.
`dump_render.debugCoverage` lists the supported paths for the selected view;
all other paths are explicitly unsupported. Terrain and grass expose albedo,
cascades and separate diffuse IBL, specular IBL and direct light. Terrain's
AO is unsupported (it does not bind screen AO). Scatter exposes albedo and
screen AO; impostors and custom shaders expose albedo. Physical materials
expose albedo, cascades and screen AO, but not separated lighting terms.
Water exposes screen AO; sky, particles, ribbons and 3D fluids use the
unsupported checker. Meshlet foliage uses the standard material shader.
Decals are part of the receiving standard/physical surface, so their albedo
and lighting follow that surface's diagnostic. Terrain-specific modes are
unsupported on non-terrain surfaces. Sprites, 2D fluids, lines, points and UI
do not implement shading diagnostics and are named as unsupported in the
dump. The physics, nav, bounds and lod modes remain overlays over beauty:
only colliders, navigation, mesh instances and og.Model respectively are
their subjects; other paths do not claim overlay coverage.

Levels of detail, reflection probes and screen-space reflections (depth
S6).

- **Mesh LOD.** The cook simplifies every model mesh with meshoptimizer
  into levels, `lods 1 0.5 0.25` by default (import settings `model {
  lods … lodError … }`; `lods 1` turns it off). A glTF's own `MSFT_lod`
  levels are kept as they are. A level is a range of the primitive's
  indices, and the runtime gives each coarse level a compact mesh of its
  own. The cook measures every level's error: the largest distance
  between the level's surface and the full mesh's, both ways (each
  corner of one against the other's nearest triangle, og-render's BVH),
  in the model's units, written as the level's `error` row and listed
  with the triangles per level by `og cook --report` (`lodErrors`). The
  renderer picks a model entity's level by that error projected to the
  screen (`og_render::lod`): the coarsest level whose error covers less
  than one pixel of the render target's height, at the near side of the
  model's bounding sphere, is drawn, so a 0.7 m lantern keeps its full
  mesh until it is far enough that its 3 cm level-1 error is sub-pixel,
  where a 6 m tree drops a level ten times nearer. (Depth S6 picked by
  the triangle fraction a level kept, level `k` below `1.2 × fraction`
  of the view height, which drew voxel props at a quarter of their
  triangles while they still filled a third of the screen and left the
  measured error unread; debt round 2 replaced it.) The previous level
  is kept within 15 % of a boundary (hysteresis), so a tree at a
  threshold does not flicker. The projected error is scaled by
  `og.Model.lodBias × RenderSettings.lodBias` (2 keeps detail twice as
  far), and a bias of 0 draws the full mesh. `dump_entity` reports the
  level drawn (`render.lod`), and `debugView lod` tints every model by
  its level: green, yellow, orange, then red.
- **Reflection probes.** An `og.ReflectionProbe` is a box of the world,
  centred on its entity, whose lit surfaces reflect a cube captured at
  its centre instead of the environment map (`probes.rs`). Each capture is
  six 90° views drawn with the lit pipelines in a `reflection-probes`
  graph pass before the main pass: the opaque meshes, skinned meshes in
  their current pose, terrain, scattered copies within the box's half
  extent of it, the sky, water and 3D fluids (as the main pass draws
  them: a face with either first draws its own depth prepass, the views'
  `depth_prepass`, then the fluids' depth, thickness and smoothing
  through the face camera into face-sized targets, and ends its pass
  after the sky to copy itself, so the water and the fluids refract the
  face drawn so far and the water darkens and foams by the depth below,
  and on the full tier reflects the face in screen space over the same
  depth and copy; no planar reflection, and the copy holds the opaque
  face as the frame's holds the opaque scene (og-render README,
  Reflection probe faces);
  `WaterRenderer::face_draws`, `Fluid3dRenderer::camera_frame`, the
  targets one set per face size in `Probes`, made the first time a capture
  needs them and drawn only while one is due), then transparent meshes
  sorted back to front from the probe, the particle systems (billboards facing each face, through
  its own `ParticleViewGpu`), scene lines and points and world sprites and
  text, on every layer (sprites unlit by 2D lights and without glow, soft
  particles fading against the face's own depth prepass, drawn for them
  while a capture is due); lit
  by the sun and every local light, its own reflections
  the environment's (one bounce). It is then copied and GGX-prefiltered like the
  environment (`ibl.rs`). The lit shaders box-project the reflected ray
  onto the probe's box, so a room's walls reflect where they stand, and
  blend the probe into the environment over `blendDistance` at the box's
  faces. Probes cost no texture slot in the lit pipelines (a choice made
  when a skinned lit pipeline had to fit sixteen textures). A draw inside a probe's box binds that
  probe's frame group instead, whose uniform carries the box and whose
  two IBL cubes are the environment's and the probe's. The probe is
  chosen per instance (`probe_select.rs`): the smallest box holding the
  centre of its bounds. The instance sort keys on it, so one mesh and
  material in two rooms is two batches, each binding its room's probe (a
  per-instance index would need every cube bound at once, as a cube
  array). Skinned instances choose by their posed bounds:
  each joint's bind-pose box of the vertices it moves (made once per
  mesh, `skin.rs`) through the frame's joint matrices, so a figure whose
  root joint walks it into the next room takes that room's probe;
  terrain levels, scatter chunks, water surfaces and the 3D fluids per
  draw, by the box holding their centre or else the smallest box their
  bounds reach. Every lit shader (standard, physical, custom, terrain,
  scatter, water, 3D fluid) samples through `frame.wgsl`'s
  `ambient_light`/`reflected_light`. At most `MAX_REFLECTION_PROBES` (8)
  probes are used, nearest the camera first.
  - A `baked` probe is captured once. `og bake probes [project]` renders
    the baked probes on the native host and writes each into the cook
    cache as an environment blob (`cooked/probes/<key>.ogb`, kind `probe`
    in the manifest; the cook keeps it while the file exists). The key
    hashes the probe's position, box and face size and its room's content
    (`probe_content.rs`: what the capture draws in its box, by value:
    meshes, materials and their texels, transforms, terrain, scatter,
    lights, the sun and the environment), so a changed room is a new key.
    The runtime loads the blob when its key is in the bundle; otherwise
    the probe draws its capture on its first frame (lazily). Beside each
    blob a record (`cooked/probes/<key>.ogt`) names the model assets in
    the room with their cooked hashes and, when game code built part of
    it, the hash of `src/`; `og doctor --project` reports a bake whose
    assets or code changed since as `stale-probe`.
  - A `realtime` probe is drawn again every `interval` ticks.
- **SSR.** `PostEffects.ssr` (with `ssrMaxDistance`, `ssrThickness`,
  `ssrSteps`, `ssrFade`, `ssrIntensity`, `ssrMaxRoughness` and
  `ssrResolution`) is stochastic screen-space reflection after AMD
  FidelityFX SSSR and its reflection denoiser (MIT,
  `third_party/fidelityfx-sssr`) and Frostbite's SSR (Stachowiak 2015)
  (`ssr.rs`, `ssr.wgsl`). One graph pass, `ssr`, between the prepass (and
  the AO passes) and the main pass, records compute passes: the
  min-max depth pyramid of the prepass's view depth; one ray a pixel (or a
  2x2 block at half resolution, a different pixel each frame) drawn from
  the GGX distribution of visible normals for the pixel's roughness and
  marched through the pyramid in (uv, 1/z), a ray more than the
  thickness behind a cell's farthest surface crossing it on that level
  (Uludag 2014's min-max Hi-Z) instead of stopping on the first pixel
  behind a thin occluder, and a ray coming out from behind an occluder
  into a surface it was in front of when it went hidden hitting that
  surface past the occluder's edge (rather than a hole), with FidelityFX's hit
  validation (the lobe's grazing rim is never sampled, Stachowiak's
  bias, and each pixel's frames walk the R2 sequence); the resolve, where
  each pixel weighs its neighbours' rays by its own lobe over each ray's
  pdf; FidelityFX's edge-stopping prefilter; and the temporal pass, the
  denoiser's reprojection (a mirror follows its reflected point's
  parallax, a rough surface its own motion, a disoccluded history is
  dropped) and temporal resolve (history clipped to the neighbourhood's
  box, whose variance is averaged over four frames as SVGF's so a rough
  lobe's sparse tail keeps its spread, and which a still history centres
  on the neighbourhood's mean over the last four frames, NRD's fast
  history, so a still rough reflection converges clean and one whose
  object moved away clears in a few frames; up to 32 samples on rough
  surfaces, 64 when still). A hit reads the previous frame's
  lit image (TAA's history, or the main pass's copy that screen-space GI
  reads too), so a frame after a cut has none. The prepass's depth
  target is RG32F, and its second channel carries the gloss and
  metalness the lit pass shades: the factors times the roughness and
  metalness maps, a terrain's blended layer roughness, a scattered copy's
  material. The result, the reflected radiance premultiplied by the
  share of the lobe the screen resolved, is a read-only storage texture
  in the frame group (binding 13, no texture slot), and `frame.wgsl`'s
  `ssr_reflected` blends it over the radiance the reflection probes, the
  probe GI's specular and the environment give, before the split sum, in
  every lit shader that writes the prepass's reflectivity (meshes,
  physical materials, terrain, grass, impostors, scattered copies,
  custom lit materials): diffuse light is untouched, and the probes and
  the sky show where a ray leaves the screen, hits the sky, misses or the
  surface is rougher than `ssrMaxRoughness`. `Frame.ssr` switches it: on
  in the main camera's uniform, off in a view's, a probe face's and a
  planar reflection's. Opaque surfaces of a perspective 3D camera only
  (`RenderCapabilities.ssr` is false only without a renderer).
- **Screen-space GI (R3).** The lazy resource `og.GlobalIllumination`
  (`enabled`, `intensity`, `maxDistance`, `thickness`, `steps`, `rays`,
  `occlusion`, `resolution`, and `allowed`, the quality switch
  `og.render.applyQuality` writes on a world that has it), registered on
  first use so a world without it keeps its type table and hash, draws in the AO passes' place (`ssgi.rs`,
  `ssgi.wgsl`): the AO targets become RGBA16F, r the occlusion, gba the
  bounce, so the lit passes read it through the effect group's AO
  texture and no lit binding is added (the lit stages sit at 16 of 16
  sampled textures). Each pixel casts cosine-weighted rays from its
  prepass normal and marches them through the view depth, each cut where
  it leaves the view frustum and its `steps` spread over what it crosses
  on screen (perspective-correct, finer near the pixel), so a ray reaches
  across the screen rather than a fixed world distance (`maxDistance`,
  16 m, caps it). Marching 4 m on world-space steps it gathered 0.40 of
  a path-traced gi-room's bounce; across the screen it gathers 0.65,
  of the 0.72 to 0.80 the screen shows at all (the rest is off screen
  or behind what it shows). A hit on a
  surface facing the ray reads the previous frame's lit image where the
  hit pixel's velocity puts it: TAA's history under TAA, else a copy of
  the main pass's image kept between frames. One bounce a frame, many
  over a few. Under TAA the rays' noise moves every frame and TAA's
  accumulation is the denoiser; the blur and the half-resolution
  upsample are the AO's, over four channels and weighted by normals.
  The bounce adds to the diffuse ambient light times the albedo; the
  rays that hit take their share of the ambient light, or with
  `PostEffects.ao` its term does. Scattered copies keep the AO alone.
- **Probe GI and sky occlusion (area 1).** `og.GlobalIllumination.method`
  `probes` (or `both`, with the screen rays over it) lights every lit
  surface's ambient light from DDGI probes (Wicked Engine's, MIT,
  `third_party/wicked-ddgi`; `gi_probes.rs`, `gi_*.wgsl`).
  - **Probes.** Cascades of 16 × 8 × 16 probes scroll with the camera,
    `probeSpacing` apart in the first cascade and twice as far in each
    next one. Each is centred 0.3125 of its extent ahead of the camera,
    so what the frame shows is lit by the finest cascade. Centred on the
    camera, half the grid lay behind it, and a room's far wall took its
    colour bleed from probes twice as coarse. Cascade c relights 1/2^c
    of its probes a frame.
  - **Tracing.** Each probe traces `probeRays` rays in software through a
    two-level BVH of the scene, simplified to at most 8192 triangles a
    mesh and 129² samples a terrain (`gi_scene.rs`). WebGPU has no
    hardware ray tracing. A hit is shaded with the sun, one shadowed
    local light, the hemisphere light, emission and the probes' previous
    bounce.
  - **Probe data.** Each probe keeps deringed L1 harmonics and a 12²
    octahedral map of distance moments. Lit fragments read 8 probes a
    cascade, weighed by a Chebyshev visibility test, so light does not
    leak through walls. A probe whose rays find it inside geometry is
    left out and moved out through the nearest back face; its next
    update starts its light and distances again, so what it saw inside
    does not hold it dark while its estimator catches up (a probe on a
    wall's plane left a dark spot on the wall for dozens of frames).
  - **Sampling (`gi_sample.wgsl`).** A lit fragment gathers the 8 probes
    around it in each cascade once, into their weighted harmonics, and
    evaluates them along its normal for the diffuse light and along the
    reflected ray for the reflections' occlusion. The ring buffer's slot
    and key are worked out for the base corner alone. On Metal at 1080p
    (gi-room-probes) the main pass went from 7.98 ms to 3.70 ms against
    1.48 ms with no GI: a second gather for the reflections took 3.2 ms
    and the corners' integer division 1.5 ms.
  - **Bindings.** The probes share the light volumes' storage buffer
    (frame binding 10): the lit stages have no texture slot left, and
    nothing beyond the default limits is requested. The effect passes
    bind it too (binding 7), for the screen GI.
  - **With the screen GI (`both`).** The screen rays reach one probe
    spacing at most, and their hits replace the probes' light from the
    directions they hit: the screen GI's pass gathers the probes and
    takes their L1 light along the hits (h c0 + 1.5 c1·s over π, with h
    the hits' share and s their mean direction), not an even share of
    every direction's, which counted a bright wall's light twice (1.11 to
    1.21 of a path-traced gi-room). L1 is too smooth to say how much of
    the probes' light came from far hits, so long screen rays still
    overcount (1.07 at 2.5 m with exact probes); within a spacing the
    sum is within 3% of the probes' own. Both now give 0.87 to 0.97 of
    the reference, as the probes alone (0.85 to 0.97).
  - **Sky occlusion.** The sky reaches a point only through the probes'
    rays that miss, so a roof, a room or a street of buildings hides it.
    Reflections outside every reflection probe are scaled by what the
    probes see along the reflected ray (`gi_reflected_from`).
  - **GTAO bent normals (XeGTAO, MIT, `third_party/xegtao`).** With
    `PostEffects.ao`'s GTAO on, the AO targets become RGBA16F and carry
    the world bent normal in gba. The lit passes read the probes along
    it. The screen GI's bounce and the contact shadows' masks use the
    same channels, so either one turns the bent normals off.
  - **Radiance Cascades** were measured against the probes and not
    ported (`tests/evidence/gi-area/NOTES.md`,
    `tests/evidence/gi-rest/NOTES.md`). In a BVH a short interval still
    costs 57% of a full ray, so world-space RC needs about 6× the probes'
    tracing. Lighting from a 0.2 m emitter was no closer to the reference
    than DDGI at the same spacing (0.61 against 0.60 mean relative error).
    Split RC, the newest 3D variant, needs hardware rays and still leaks
    below its spacing. Screen-space RC reaches no further than the
    screen GI.
  - **Leaks and cost.** Walls a twentieth of the probe spacing leak under
    0.1% of the light outside, so the visibility is complete. RTXGI is
    under the NVIDIA RTX SDK licence, so it is cited, not ported. On
    Metal at 1080p, gi-room-probes' tracing takes 1.6 ms and the lit
    pass's sampling 2.2 ms (Sampling, above).
- **The live sky (R3).** While the background is the per-pixel sky
  (`Background::Procedural`: og.Sky, og.DayNight, og.PhysicalSky,
  og.Clouds) and no environment map replaces it, the image-based light
  and the reflections' fallback (surfaces outside every probe, SSR's
  misses, water's) follow the sky as drawn: Unreal's real-time sky light
  capture, ported (`sky_capture.rs`). `fs_sky_look` draws the sky from
  the camera into a 128² cube through the face cameras' frame uniforms
  (stars off), the reflection probes' filter turns it into the mip
  chain, the irradiance and the GGX-prefiltered levels in cubes of its
  own, and a copy puts those into the cubes the frame binds once they
  are whole: no binding is added and a frame never samples a half-made
  sky. The work is spread over eight frames (`FramePass::LiveSky`, first
  in the graph and timed as `live-sky`): two of faces, one of mips and
  irradiance, four of prefiltered levels, one copy; the first frame of a
  live sky does all eight, and a change to the sky reaches the light
  within two captures (sixteen frames). The capture's clouds take the last capture's
  irradiance as their ambient light. The IBL generator's own sky (the
  1024-byte parameter slots, the BRDF LUT, environment maps) is
  unchanged and draws the first frame's lighting.
- **Physical solar radiance and cloud view fog.** The physical sky adds
  the atmosphere-attenuated solar radiance after scattering, retaining
  limb darkening and filtering the disc boundary over one pixel's angular
  footprint. Its aureole comes from scattering and bloom, not a feather
  proportional to its radius. Only the scattered sky retains the look's
  saturated-colour shoulder; the sun reaches bloom and the tonemapper in
  scene-linear HDR (`third_party/hillaire-sky.md`). The sky output is
  bounded to finite RGBA16F storage; the HDR output mapper bounds it to
  the display peak. Cloud scattering samples receive the same distance
  fog as meshes at their distance; the height fog's column, which a
  cloud above it shares with the sky beside it, they take by `Fog.sky`'s
  share as the sky does.
- **Aerial perspective and far clouds.** Under the physical sky the air
  between the eye and a cloud is integrated with the sky's own
  scattering (Hillaire 2020 §5.5, `sky_scatter.wgsl` `air_path`, at the
  cloud's transmittance-weighted depth): the cloud's light is seen
  through the air's transmittance and the air's in-scattered light it
  hides behind it is replaced by the same air's in front, with the sun
  shaded by the cover (`Clouds::light`) and, under a closed deck, the
  sky's light by the deck's base. Past the march's reach (30 km of
  marching from the layer's entry, 60 km to the entry at most) the layer
  is drawn as its mean: a ray meets cloud with probability
  `1 - (1 - cover)^(path / cell)`, lit by the same sun, multiple
  scattering and ambient as the march, behind the marched part and
  instead of it past 60 km, so horizon cloud banks continue to the
  horizon under the same fog and air. The gradient sky keeps its 27 km
  fade (it has no air to integrate).
- **Lens flare through clouds.** With clouds drawn over the procedural
  sky the flare's sun is the one above them and its occlusion pass reads
  the disc as the sky drew it against the disc's expected luminance
  (`lens_flare.wgsl` `fs_flare_occlusion`), so a cumulus passing over the
  sun dims the flare by its transmittance.
- **Realtime light volumes (R3).** The lazy component
  `og.RealtimeLightVolume` (`updatesPerFrame`, `intensity`) beside an
  `og.LightVolume` relights that box's probes while the game runs
  (`probe_grid.rs`, `probe_grid.wgsl`). A probe is relit by a 16² cube
  capture drawn through the reflection probes' capture path (every
  light with its shadows, the sky, emissive and lightmapped surfaces),
  then projected onto L1 harmonics by a compute pass straight into the
  light volumes' storage buffer (frame binding 10), after the baked
  volumes; its header entry carries bit 21 of the packed counts. No lit
  binding is added. Up to eight probes a frame over every volume, in a
  stride order that spreads them over the box; the captures sample the
  probes, so each sweep adds a bounce. A volume is sampled once swept
  (its first sweep at eight a frame); until then, and while the quality's
  GI switch is off, its box takes the baked probes or the environment.
  `ambient_irradiance` blends the realtime volumes over the baked ones
  over the environment by the same edge weight; lightmapped surfaces keep
  their lightmaps. The realtime probes relight reflections as well
  (`volume_reflected`, wrapping `reflected_light` in pbr_fs and
  physical): the reflection is the probes' L1 irradiance along the
  reflected ray, a soft image of the room; inside a reflection probe's
  box, smooth surfaces keep its capture (drawn with the probes' light),
  blurring into the probes' as roughness rises. The projection derings
  each channel (|c1| ≤ c0), so the irradiance is nowhere negative and no
  channel clips alone into a hue shift or a hard terminator. Baked
  volumes leave reflections as they were. A
  realtime volume is whole up to its faces and fades out over a spacing
  beyond them (a baked one fades in over its outer spacing), so the room
  surfaces just outside its probes take them. The low preset's GI switch reaches a world that names
  `og.GlobalIllumination` only after the preset was applied: the preset's
  switch waits in the presentation state (`FxState::gi_preset`), never
  the world's.

Render targets, views, layers and custom shaders (depth S8).

**Render targets.** A render target (`targets.rs`) is two entries in the
scene's texture table: its colour and, optionally, its depth. Any
material slot, sprite or UI image samples it like any other texture, and
there is no separate texture kind for a game to learn. Snapshot and
restore keep the targets. A restored renderer draws every view on its
next frame, so a target drawn every few ticks never shows the cut run.
- `ldr` (Rgba8 sRGB) holds the view's finished image.
- `hdr` (Rgba16Float) holds its linear radiance. The frame that shows it
  exposes it, so a mirror matches the room.
- The depth texture is drawn on every tier. GLSL ES 3.0 cannot load a
  depth buffer (a multisampled one least of all), so the depth never comes
  from the view's depth buffer: the view draws its opaque meshes, skinned
  meshes and terrain again through the prepass pipelines into a
  single-sampled linear-depth colour target (RG32F, as the main
  prepass's `lindepth`), and the resolve normalises it from near to far.
  One path on every backend, so the native gfx tests run what the browser
  runs.
  `RenderCapabilities.renderTargetDepth` is true with a renderer.

**Views.** A view (`views.rs`, `og.CameraView`, `og.Mirror`) is a
secondary camera. Each is a timed graph pass, `view-<i>`, drawn before the
main pass in `order`, so a target shows this frame's image. Each view has:
- its own frame and effect uniforms;
- light clusters built from the frame's lights;
- its HDR and depth attachments, kept while the view exists;
- a resolve into its destination. A target destination is exposed and
  tone mapped for ldr, and copied linear for hdr. A viewport rect draws
  after the post chain, under lines, the overlay and the UI.

A view draws what the main pass draws, from its eye and on its layers:
opaque and transparent instances (sorted from its eye), skinned
instances, terrain, sky, particles (billboards facing its camera, through
its own `ParticleViewGpu`), scene lines and points (its own
`LineUniform`) and world sprites and text (`sprites::build` once per view
with its mask, the instances appended after the main camera's), and
trails, ribbons and beams (the frame's strips emitted again facing its
eye, `Trails::emit`, after the main camera's in the strip buffer). Its
lit sprite layers multiply with its own 2D light map, the frame's 2D
lights drawn through its camera into a target of its size times
`Lighting2d.resolution` (`Light2dRenderer::write_view`), and its 2D
fluids are composited in its sprite order from field targets of its own
size, the frame's kernels drawn through its camera. In a frame with soft
particles every view runs its depth prepass (the one a target's depth
texture takes) before its pass, and its particles fade against it. With
`PostEffects.glow` its glowing sprites draw into a glow chain at half its
size, run through the frame's own bloom filters (13-tap downsamples, tent
upsamples, `bloom_chain.wgsl`) and added onto its image before the resolve
at the weight the frame's tonemap gives its glow (`views_glow.wgsl`): the
main frame's glow, the same halo, without the rest of the bloom.
Screen sprites, the UI and the post chain are the main frame's. Of the post chain an `ldr` target or
a viewport carries exposure and the tone curve (the resolve); an `hdr`
target carries linear radiance for the frame that shows it to expose.
The cost is explicit: a view draws everything its mask admits, so its
`layers` and `every` are the knobs. Under GPU culling (`cull_gpu.rs`)
each view is a pass of its own beside the main camera's, its shadow maps
and the planar reflection: its planes and its layer mask, its opaque
instances drawn indirect from its region as the CPU list would draw
them. Its transparent instances, sorted on the CPU from its eye, and its
skinned ones keep their CPU lists, appended to the instance buffers.
Skinned instances and baked crowds are culled by their posed boxes on
both paths: the joints' bind-pose boxes moved by the palette (a vertex
lies in the hull of its joints' moves of it), a crowd's clip's over all
its frames; under GPU culling the camera's pass, its occlusion test
included, the planar reflection and every shadow pass take them with the
instances, and their records are compacted per pass for the skinned
draws.
Crowds (`RenderScene::crowds`, og-render's `crowd.rs`, what
og.CrowdAnimation members draw as) are the GPU-driven path's skinned
workload: per culled pass a compute pass (`crowd_cull.wgsl`) tests each
member's clip box against the pass's planes (the camera's also against
the Hi-Z pyramid), picks its level from the camera (lod.rs's rule), and
compacts the survivors by a deterministic prefix sum into indexed
indirect draws that blend the clip's baked frames in the vertex stage.
A crowd's last level may be an impostor (`CrowdImpostor`): cards of its
full mesh baked on the GPU per view around the vertical and per frame of
its clip (`crowd_impostor.rs`), shared by crowds of the same meshes and
coloured per part by each crowd's materials, lit, depth-written and cast
into the shadow maps as the meshes are. Without
`INDIRECT_FIRST_INSTANCE` (`RenderInfo::gpu_crowds`, the capability
`gpuCrowds`), with secondary views or probe faces, crowds take the CPU
lists: the same members at the same levels.
- `every` skips ticks.
- A mirror reflects the main eye in its plane. It draws through an
  off-axis frustum fitted to the quad's corners (Kooima), with the near
  plane on the glass.
- The id pass (`id_pass.rs`) draws from any camera on its layers
  (`render_ids_from`, `view_id_cameras`). The probe's `describe` lists the
  main camera's entities, each naming the views that also show it
  (`views`), and each view with what its own pass shows (§3.4).

**Layers.** Layers are three.js masks: bit n is layer n, everything
defaults to 1, and a camera draws an object when `a & b != 0`.
`RenderScene::instance_layers` and `skinned_layers` split runs and batches
by mask; `sprite_layers`, `line_layers`, `point_layers` and
`particle_layers` carry the other drawables' masks, and every camera (the
main one with `camera_layers`, each view with its own) tests them in the
renderer. og-runtime fills them from the `layers` of og.Mesh, og.Model,
og.Sprite, og.Lines, og.Points and og.ParticleSystem.
`og.render.raycast` makes the same test against the same per-entity masks.
Shadow maps take every caster.

**Full custom shaders.** `type shader` with `mode full [lit|unlit]`
(`custom.rs`, `shaders/custom_*.wgsl`) has its own `vertex` and
`fragment` entries over a fixed binding contract:
- `camera()`;
- the model matrix, with the skin folded in and morphs applied;
- `Params`;
- `tex0` to `tex7`, the material's eight texture slots;
- the `standard_vertex`, `standard_surface` and `noise3` helpers.

The modules keep the built-in entry names and vertex buffers, so they
draw through the hooked materials' pipelines. The prepass, every shadow
pass, the water's reflection and every view run the user's vertex
entry, and cut by its alpha. The cook validates each tier's modules
(WGSL, SPIR-V, MSL, GLSL ES) with naga, as it does for hooks.

**Custom post passes.** A `type post` asset (`post_custom.rs`, an
og.PostPass blob) is a WGSL `post` function. It reads the previous pass,
the lit image, the prepass's depth and normals, and its parameters.
`PostEffects.custom` is a list field of up to eight passes (`List.Post.8`
in the IDL: og-render's `MAX_CUSTOM_POSTS`,
`RenderCapabilities.maxCustomPosts`), which run in its order, a 0 in it
skipped, as `custom-post.<i>` after motion blur and before exposure, bloom and the
tone curve. Each pass compiles in an error scope. One the device refuses
is skipped and reported (`og.render.postErrors`); the frame draws as if
the pass were absent.

**2D lighting (R11).** `light2d.rs` draws the light map that lit sprite
layers multiply with. It is cleared to the ambient, and each `Light2d`
adds an instanced quad over its radius. One strip per receiver light
mask sits side by side in one target (`LightSlots`, at most four).
- **Shadows.** Each shadow-casting light owns a row of a polar shadow
  map (`light2d_shadow.wgsl`, `Lighting2D.shadowAngles` texels wide,
  RG32F). The row is traced once a frame against the edges of the
  occluders that reach the light: boxes and polygons, concave ones too.
  Each texel keeps the first hit and the far side of the first occluder
  met, so a pixel inside an occluder is not shadowed by it. The cost per
  light-map pixel does not grow with the occluders. The light pass
  samples the row with PCSS: a blocker search over the light's angular
  size, then a filter whose width is the penumbra at the blocker's
  distance. The column wraps at ±π.
- **Normal maps.** A texture with a `<id>#normal` (`normal auto`, made
  by the cook from the silhouette in `normal2d.rs`, or `normal "file"`)
  is linked by the runtime (`RenderScene::set_texture_normal`). While
  any texture has one, the light pass also writes `lightdir` (`fs_dir`):
  each light's luminance-weighted unit direction, towards the light at
  `Light2D.height` above the plane. Its draws (`sprite_normals.rs`:
  `fs_normal`, `fs_mesh_normal`) light by
  `ambient + (L − ambient)·(n·d + ½)/(d.z + ½)`, with `d` the mean
  direction and `n` the map's normal turned into the world by the uv's
  screen derivatives (so turned, mirrored and bent receivers are lit
  right). A flat normal gives exactly `L`. Every other draw keeps the
  plain sprite pipelines, and a frame without normal maps makes no
  `lightdir`.
- **`og.render.light2dAt`** (og-runtime `light2d_at.rs`) answers the
  light at a point for game rules: the same falloff, cone and fade, hard
  shadows through the same outlines, in og-core's deterministic math, so
  the answer is the same bits on every host. The GPU's `atan2` and `pow`
  are never its reference.

Baked lighting, volumetrics and area lights (depth S7).

- **Baked lighting** (`og bake lightmaps [project]`, the og-bake crate).
  The bake is a native CPU tool with no GPU: a progressive path tracer
  over the scene's models and its directional, point, spot and rect
  lights, with a BVH and a fixed seed, so a bake with a given `--samples`
  is repeatable. Entities marked `static true` in a scene get lightmap
  UVs from xatlas, charted over their world-space triangles at one texel
  density (`--texels-per-metre`). Each texel stores the light arriving
  there: bounces off the scene, the sky and emissive surfaces. A light's
  direct term is never baked, because the renderer draws it every frame
  with its shadows; the bake adds only what forward shading cannot. Each
  `og.LightVolume` box gets a grid of L1 spherical-harmonic probes from
  the same tracer. The bake writes `<scene>.lightmap.ogt` (the remapped
  parts, the probes, and the scene and model hashes) and
  `<scene>.lightmap.ktx2` (RGBA16F array layers) beside the scene. The
  cook turns them into an `og.Lightmap` blob, listed in `cook --report`.
  When a scene is instantiated, the runtime loads its bake if the hashes
  match. Static models then draw lightmapped copies of their parts: a
  second vertex stream, with their own scene's pages at group 3
  (`lightmap.rs`, `lightmap.wgsl`). Every instantiated scene's bake draws
  at once: each scene's pages are one texture array with its own bind
  group, and each lightmapped batch binds its scene's, so scenes loaded
  together or streamed in beside each other all keep their bakes. A bake
  stops drawing while its static models are all despawned. Every other lit fragment inside a volume takes its
  ambient light from the probes, trilinear between eight and faded over
  one spacing (frame binding 10, `ambient_irradiance`). Without a bake, or
  with a stale one (the scene's source text or a static model's geometry
  changed since the bake; a re-cook that keeps the geometry keeps the
  bake), the scene renders unbaked with the environment's
  ambient; `og.render.bakedLighting` gives the reason and `og doctor
  --project` warns `unbaked`. `RenderSettings.bakedLighting` false draws
  every scene unbaked. A static mesh draws its bake in every lit batch:
  standard, lean and compact (`vs_lightmap_compact`), texture-array
  (`fs_lightmap_array`), material-shader (its module carries the same
  entry points) and physical (`lightmap_physical.wgsl`).
- **Projection painting** (`og bake projection <step> --run <dir>`,
  `og-bake/src/projection`). A native CPU tool, like the lightmap bake,
  that turns a gray level glb into a textured one by painting
  it from views: the trompe workflow (Patch-gg/trompe, Apache-2.0)
  without Blender or a GPU. The image model stays outside the engine; a
  driver (the knowledge package's `level-texturing` skill, with its
  `scripts/texture-level.ts`) runs the steps and calls
  it between `prepare` and `project`. `init` rasterises the level's
  TEXCOORD_0 into texels (world position, normal, triangle) with a BVH
  (a level without TEXCOORD_0 is charted by xatlas at one texel density
  into one page first, and the run keeps that charted level);
  `plan` picks cameras in paint order from walkable points on the
  floors (ground, then roof passes and an aerial ring) by greedy coverage
  of a probe render; `prepare` ray-casts a view, renders its clay (ambient
  occlusion and a key light) and composites the texture painted so far
  into `state.png` with `edit.png` (white where the view sees no paint)
  and a mode (`first`, `fill`, `roof`) for the prompt; `project` takes the
  model's picture, keeps the known pixels, colour-corrects and feathers
  the new ones into them, drops what the model left unpainted (unchanged,
  or flat gray over a whole patch, since some models redraw the whole
  image) and writes each visible texel whose view is better than the one
  that painted it (a texel is visible when the pixel's ray hits its
  triangle, or within 1 cm and 0.5 % of depth with agreeing normals;
  silhouette pixels count less); `holes` adds the view that sees the most
  unpainted texels; `export` pads the uv gutters and writes the level with
  the texture as every material's base colour. The run directory holds
  everything (`run.json`, `plan.json`, `texels.bin`, `paint.bin`,
  `views/K/`), so a run resumes and a view can be re-projected.
- **Volumetric fog** (`og.Fog.volumetric`, `volumetric.rs`). The fog's
  density, height fog, `scattering` and `anisotropy` (Henyey–Greenstein)
  define a participating medium up to `volumetricDistance`. On the full
  tier the medium is a froxel grid (160 × 90 × 64, slices deepening
  quadratically), handled by a named `volumetric` compute pass. The pass
  lights each cell by the sun through its cascades and by the cluster's
  point, spot and rect lights through their shadows and cookies (each
  froxel the mean of eight stratified, jittered samples over its slice's
  depth, so a shaft crossing a deep slice is integrated, not hit or
  missed column by column), then integrates front to back. The
  `raymarch` method, a choice on any device, marches each pixel's ray in
  24 steps, each lit by the sun and by the first 8 point, spot and rect
  lights of its cluster (`RAYMARCH_LIGHTS`) through their shadows and
  cookies. `RenderCapabilities.volumetricLights` reports the per-sample
  bound (64 in the froxel grid, 8 in the raymarch); `RenderDesc::volumetric`
  (`OG_GFX_VOLUMETRIC=raymarch`) selects the raymarch.
  `RenderCapabilities.volumetricFog` reports which method the device uses.
  Both methods blend over the lit image after the main pass as
  colour × transmittance plus in-scattered light, reading the prepass's
  linear depth.
- **Area lights** (`og.RectLight`: width, height, colour, intensity,
  `twoSided`; `area.rs`). They are clustered with the point and spot
  lights and shaded with linearly transformed cosines (Heitz et al.'s GGX
  fit, `third_party/ltc`, resampled to 32 × 32 half floats in one 16 KiB
  uniform). Their highlight on a glossy surface is the rectangle's shape.
  With `castShadow` one takes a layer of the local shadow array (two when
  two-sided, counted in the shadowed lights' budget): a 150° perspective
  view from its centre along its normal (`RECT_SHADOW_FOV`), read as
  percentage-closer soft shadows (`rect_shadow`): 16 taps find the mean
  blocker distance, then 24 bilinear taps, each pixel's spiral turned by
  a hash of its position, filter over the penumbra the rectangle casts,
  half-size × (receiver − blocker) / blocker. A contact shadow is sharp
  and a far one soft; the view is from the centre, so a caster close to
  a large panel is shadowed as from its centre with its penumbra widened
  by the panel's extent, and receivers past 75° from the normal stay
  unshadowed. Terrain, scattered copies and 3D fluids shade them with
  LTC and their shadow too (and take point and spot cookies), through one
  shared `surface_local_light` in `area.wgsl`, and so does a full custom
  shader's lit surface (`custom_lit.wgsl`).
- **Light cookies** (`SpotLight.cookie`, `PointLight.cookie`, a cube of
  six faces stacked in one texture). A small pass draws each cookie from
  its uploaded texture, in whatever format was cooked, into layers of the
  local shadow array past the shadow layers. Each texel is packed as an
  8:8:8 integer in the R32Float array, so a cookie takes no texture slot
  on either tier. At most 24 layers are used (`MAX_COOKIE_LAYERS`,
  reported as `RenderCapabilities.maxLightCookies`): one per spot cookie,
  six per point cookie, a texture shared by lights of one kind once.
  Cookies past them, in their lights' entity order, are not drawn; the
  frame warns once in the log, naming the lights that shine without
  theirs (`og_render::cookie_layers`).

**three.js parity for ports** (the Grand View Garden fork merge).
A game moved from three.js needs its look kept, not reinterpreted, so a
set of opt-in fields reproduces what three.js and its common passes draw;
each is off at its default and leaves every existing frame unchanged.
Lighting: `Environment.hemiSky/hemiGround/hemiIntensity` is
HemisphereLight (diffuse-only, darkened by AO); `DirectionalLight.shadowBox`
replaces the three cascades with one fixed box around a focus ahead of
and below the camera (`shadowFocus`, `shadowFocusDrop`), the orthographic
shadow camera such games fit around the player, with three.js' `bias`
and `normalBias` (`shadowBias`, `shadowNormalBias`). Post:
`PostEffects.aoMethod screen` is the depth-only screen-space AO of
three.js-era post passes, `aoRadiusScale` grows the AO radius with
distance, `aoDirect` lets the AO darken direct and emitted light too
(and, above 0, see and darken water surfaces, as three.js' post AO did;
at 0 water stays outside the AO), `sharpen` is a CAS pass after the
output, and
`RenderSettings.bloomRadius/bloomContrast` switch the bloom to
UnrealBloomPass. Water: `Water.geometry` draws over any mesh (a stream
or a pool) instead of the grid, `f0`, `reflectionBias` and `glint` are
Water.js's Fresnel floor and sun glint, and `normalMap/normalScale/flow`
are Water.js's drifting and Water2's flowing normal-map ripples.
Materials: `alphaToCoverage` (with MSAA) turns alpha-tested foliage into
sample coverage, scatter included; material shaders get `Hook.eye`,
`Hook.front`, `planar_reflection(p, shift)` (the frame's planar
reflection, for a game's own water or floor, with
`RenderOptions.planarReflections 2`) and
`screen_ao(p)`.
`Terrain.visible false` keeps a terrain as ground for its scatter only,
and `TerrainScatter.castShadows` turns the copies' shadows off. og-ui's
style `ramp` takes a fifth number. The cook's `color_textures uastc`
(project.ogt, §2.5) packs colour textures as UASTC where ETC1S's 4×4
block loss shows on foliage and bark, overridable for the web export.

**Render origin (R12).** The frame is drawn relative to a point near
the camera, so the GPU's f32 arithmetic works on numbers within a few
kilometres however far the world reaches (`og-render/src/origin.rs`).
Extraction builds the frame in the world, then moves it to the render
origin, the point of an 8192 m grid nearest the camera
(`extract::render_origin`): a power-of-two grid, so `p - origin` is exact
for every position near it, and a camera within 4096 m of the world's
zero draws at it, exactly as before. `RenderScene::rebase` moves every
world-space position the frame draws (instances and their previous
transforms, cameras and views, lights, sprites, text, particle systems
and colliders, trails, ribbons, beams, scene lines and points, decals,
probes and light volumes, water, terrains, fog heights, 2D lights,
fluids) and records the origin in `RenderScene::origin`; screen-space
sets and the overlay stay. The camera's look comes from its rotation, not
from a target `position + forward` rounded at its distance from the
world's zero. What the renderer keeps between frames moves with the
origin, so a move is invisible: TAA's history and the previous
view-projection (`origin::shift_matrix`), world-space particles (a shift
dispatch on the GPU, the slots on the CPU) and trail histories. Patterns
laid in the world add the origin back, in f64 on the CPU and without the
whole repeats, so they stay put: terrain layer uvs, the lake's normal-map
layers, caustic cells and particle turbulence (whole lattice cells plus
the rest), the stylised surface origin, the shadow cascades' texel grid,
and the wind phase of scattered copies (placed in the world, then moved,
so their bits hash the same). Material hooks read `h.origin` and full
custom shaders `camera().origin` for patterns of their own. Rendering
stays presentation: the origin never reaches the tick. A world placed
100 km out still holds f32 positions (7.8 mm apart there); keeping the
simulation precise is the tick-owned rebase of `og.WorldOrigin` (§2.4).

**Shared frames.** A native engine process can show its frames in another
process's window with no copy through the CPU: Origin Desktop's Preview,
while its built-in runner runs a Session's engine. The runner names the
window's socket in `ORIGIN_FRAMES_SOCKET`; with it set, the native host
(`og run`, `og dev`) draws in the slots' format and links its renderer
(og-render's `shared_frames`, `FrameLink`) to the socket: five shareable
textures per generation (IOSurfaces passed by Mach rendezvous on macOS,
Vulkan opaque fds on Linux, D3D12 NT handles on Windows) sent with one
`Hello`, each loop frame copied on the GPU from the offscreen target into a
free slot and announced `Ready` when the copy has finished, `Release` giving
slots back, `Resize` applied before the next frame (the play surface follows
the render target, as the web host's canvas does) and answered with
`Rehello` and the next generation. The wire is the desktop's `shared-frame`
contract (origin-agent `desktop/crates/shared-frame`), implemented here,
since neither side links the other. The engine never waits for the window:
the handshake runs on the link's thread, a frame with no free slot is not
shown there, and a window that is not listening (or goes away) leaves the
process drawing offscreen as before, reconnecting when it comes back.
Without the variable nothing changes. Synchronisation is the CPU's (the
copy's completion, the window's release after its own later frame); GPU
signals (`MTLSharedEvent`, exported semaphores, shared fences) replace it
later.

### 2.7 Animation, physics, navigation, audio, UI **(D)**

- **Animation**: glTF skins and clips, a blend tree asset (`.ogt`), state
  machines as data, root motion, additive layers, two-bone and FABRIK IK,
  blendshapes.
  The cook diagnoses each source mesh/skin binding before welding or LODs,
  including on incremental cooks. `skinDiagnostics` in `og cook --report`
  and the cook's asset notes count vertices with summed weight >0.02 on
  another limb than the dominant limb, excluding limbs with the same attach
  joint. This count is information, not a warning. Humanoid regions and
  probes use the retarget map (including `animation { rig }`): each upper
  arm turns −70° about model X, its forearm −40°, the opposite arm +10°;
  stride turns left upper/lower leg −35°/+30° and right upper leg +25°.
  Rotations compose down the hierarchy about inverse-bind joint positions,
  with +Y up and +Z forward. Without all four mapped limb roots, the root's
  longest joint chain defines the spine; side branches of at least three
  joints are regions. Each such branch and the continuing spine child at
  its fork gets a −70° probe named by its skin joint index. Only triangles
  whose posed longest edge is both >2.5× its bind longest edge and >0.08
  model units (8 cm for metre-authored glTF) warn. Warnings name the skin,
  mesh, primitive, probe and zero-based source triangle IDs. No weights,
  animations, cooked geometry or world state change; these are fixed stress
  probes, not a proof that every possible animation is free of stretching.
- **Destruction** (`og.destruction`): props are prefractured in the cook,
  never at run time. A `fracture { }` block cuts a model into Voronoi
  cells clipped by exact CSG (og-geom `fracture`, cut faces tagged with
  their neighbour and given the interior material), with optional
  children per chunk; `<model>#fracture` holds the chunks' hulls,
  hierarchy, the support graph's bonds and the anchor faces
  (og-destruct `FractureAsset`). In the tick, before og.physics, damage
  wears bond health (Blast's model of health as effective area), the
  support graph splits into islands, and Blast's stress solver (CGNR over
  the anchored islands, ported to f32 with fixed orders, the world a
  static node mirrored through each anchor face) damages what is past its
  elastic limits. An island whose membership changed is rebuilt as one
  body: static colliders while anchored, else a dynamic og.RigidBody on
  its first chunk with the rest as compound children. Bond health, chunk
  entities and queued calls are a state block. Fine debris is visual
  only: a GPU mesh-particle burst replaces pieces below
  og.Destruction.debrisVolume, which never become bodies, so the
  simulation is the same on every device.
- **Motion generation** (planned, the owner's direction of 2026-11-01;
  researched before it is built, `tests/evidence/motionbricks/NOTES.md`):
  a humanoid is animated by intent instead of hand-authored clips and
  blend trees, because an agent states where a character goes and what
  it does far better than it authors and splices clips. Game code sets
  targets (a place and heading, a style, an interaction on a prop such
  as sit, vault or pick up, whose target poses the prop's asset carries)
  and a pose-conditioned generative model in the manner of NVIDIA's
  MotionBricks fills in the motion between the character's recent poses
  and those targets, a short span at a time, chained. The model runs on
  the device, in Rust, on wgpu compute (one implementation for both
  hosts; no third-party web inference runtime), with its weights a
  cooked asset fetched like any other and sized for quality, not for a
  download budget. Its poses are presentation (§2.4): the simulation
  moves the character by its controller deterministically, the pose
  follows, and gameplay never reads it. og-anim retargets the generated
  skeleton onto the game's rig and plants the feet from the model's
  contact output. Open for the research: the model (distilled from a
  teacher or trained on licensed motion, since the target poses of the
  public release are evaluation-only), the intent API, and the crate
  boundary (inference beside og-render's device, not inside og-anim's
  tick). The model runs on the GPU only, with no CPU inference path:
  the engine targets devices with a GPU and does not natively support
  devices without one (the owner, 2026-11-02). Its weights (35 MB in
  f16) ship inside `@origingame/engine`; a game that uses `og.motion`
  cooks them as an optional pack, named in the manifest outside its
  outputs and never in the boot bundle (the owner, 2026-11-02), read and
  uploaded in the background as soon as the bundle loads (the
  coordinator, 2026-11-02). `motion.presentation(world, entity)` reports
  `loading`, `warming`, `ready` or `failed` with a reason on both hosts.
  Ready means generated frames and their current palette, never an
  authored fallback. This answer depends on GPU timing: game logic must
  not branch on it, and no component, goal, tick, snapshot or hash depends
  on it. The host's page or shell cover may wait for ready or failed while
  the agent stays in the world warming; there is no per-entity reveal path.
  `motion.status` remains the deterministic goals; `capabilities.backend`
  means weight residency, not first-pose readiness.
  The probe's `dump_motion` reports each agent's `presentation`, including
  before the model is resident. `model.startup` records wall milliseconds:
  native pipeline creation (`compileMs`), weight/mask buffer preparation
  and upload submission (`uploadSubmitMs`, not a completion fence),
  residency to first batch scheduling (`firstDispatchDelayMs`), first batch
  scheduling through readback (`firstInferenceWallMs`), time spent polling
  that batch (`firstInferencePollMs`), and readback to its first generated
  palette (`firstPaletteDelayMs`, including decode). Inference wall time
  includes GPU work and inter-frame scheduling; subtracting poll time does
  not isolate either one. Compile/upload are null on the web; unfinished
  phases are null. The 3 ms slicing is unchanged.
  The flight operator's approved root-alignment experiment and the open
  `motion.jump` vertical-arc ownership decision are recorded in
  `tests/evidence/motionbricks/FLIGHT.md`. The controller must own ballistic
  elevation; generated pelvis Y cannot silently add a second jump arc.
  Resolve that presentation-height contract before adding the jump IDL.
  The experimental flight solve operates on the whole clip: a streaming
  boundary must buffer all of its samples before solving, then read the
  solved samples by absolute age. Solving model batches independently is
  not equivalent and does not inherit its continuity guarantee. Contacts
  infeasible under the hard root remain explicit plan/model conflicts;
  neither pelvis orientation nor reach/slack may be relaxed to conceal them.
  `motion.jump` remains gated on art approval of flight recovery and the
  upright cone, besides its unresolved vertical-ownership contract.
- **Physics**: rapier3d (and rapier2d for 2D projects), fixed-tick stepping
  inside the engine tick, a kinematic character controller, vehicle later.
  A kinematic body with an `og.Velocity` moves by it each tick and its
  Transform is written back (one without stays where its Transform puts
  it): one rule for the API docs, the skills and the physics README.
- **Navigation**: navmesh baked at cook time or at runtime for procedural
  levels; path queries and crowd steering as engine systems.
- **Audio**: the mixer runs in wasm; on the web the engine Worker feeds an
  AudioWorklet through a SharedArrayBuffer ring, on native through cpal.
  Buses, spatialisation, QOA streaming, one-shot and looped sources.
- **UI**: the design of the UI as infrastructure, its rebuild (documents,
  states and motion, the UI render pass, styles v2, art slots, world-space
  UI, the agent's read, write and check tools) and its genre coverage are
  `docs/ui.md` (the owner, 2026-11-01); this bullet states what exists
  today. Immediate-mode (Clay-like layout) rendered by the engine; element
  ids are stable strings so the command channel can target them
  (`touch_move_to`, `move_to_text`, `dump_ui`). A game's UI is the game's,
  never the platform's: the engine owns its values (`crates/og-ui/src/theme.rs`,
  no platform token reaches it) and every game dresses its UI in a **style**
  (`type style`, `og.load.style`), one complete asset: the scheme, every
  palette role, the four radii (small, control, bar, panel), the type ramp,
  fonts, icons, nine-slice frames per element class, ornaments, motion and
  sounds. A look's `ui` line names it: a built-in by slug
  (`templates/styles/<slug>/`, cooked under `@styles/`; every catalogue
  preset names one) or the game's own `styles/<name>/style.ogt`. Without
  one the UI draws in `prototype`, the engine's placeholder (greys,
  near-square corners, a cyan focus ring), which `dump_ui` reports as an
  empty `style` and `og polish` flags. Contrast and accessibility bars hold
  for every style (`pack_contrast.rs`). A nine-slice's middle squeezed
  into a control shorter than its art shows the centre of the art's middle,
  squeezed at most twice, and sprites and the UI's images are sampled
  anisotropically where the device can (`RenderCapabilities.spriteAnisotropy`,
  16; 1 on a software
  rasterizer, whose filter softens even a 1:1 draw), so the outline
  through a squeezed middle is as crisp as the corners; text keeps a
  trilinear sampler (its distance field). Screen transitions, count-ups and
  bursts run by the tick in the UI state.
- **Timeline and paths** (depth S2, `crates/og-timeline`): a path is a
  curve through or controlled by points (`catmullRom`, `bezier`,
  `linear`, open or closed), measured by an arc-length table so a
  parameter 0..1 moves at an even speed; `og.path.create` makes one, a
  scene declares named ones (`path "<name>" { point x y z ... }`), and
  `og.PathFollower`, `og.CameraRail.path` and a timeline's camera ride
  them. A **timeline** is a `type timeline` `.ogt` asset, cooked to an
  `og.TimelineAsset` blob with its caption files (`.srt`, word-timestamp
  `.json`) imported: a `duration` and tracks of keys at times (camera on
  keys, a rail or following an entity; transform; animation clip; audio;
  music with fades; caption; event, a command-channel action; transition
  over og.ui's screen cover; marker). The `og.Timeline` component plays
  one through the `og.timeline` system, after the game's systems and
  tweens and before og.anim and physics. Every value is a function of the
  timeline's time alone, and that time is a whole number of thousandths
  of a tick (a tick adds `round(speed × 1000)`, a seek sets `round(t ×
  rate × 1000)`), so seeking to t and playing to t give the same world
  hash: the video producer (S3) starts at any frame. Playback state (the
  time, the camera it took, its voices) is a state block, carried by
  snapshots and the hash; a camera track owns `og.Camera` while it plays
  and hands back the camera it found when it stops or ends. The command
  channel drives it: `timeline:play:<name>` (`timelines/<name>.ogt`, on an
  entity `timeline-<name>`), `timeline:seek:<s>`, `pause`, `resume`,
  `stop`.
- **2D skeletal animation** (depth S9, og-anim `rig2d.rs` and
  `skeleton2d.rs`): a `type rig2d` `.ogt` asset in the engine's text
  (pixels, +y up, degrees counter-clockwise, `unit` pixels per world unit):
  bones parent first (rotation, skew, scale), slots in draw order (colour
  multiplier and offset), `region`, weighted `mesh`, `box` (a bounding
  polygon, drawn as nothing) and `child` (a child `armature` of the same
  rig, posed and drawn inside its slot) attachments, `ik` constraints over
  a chain of any length (one bone points, two are solved exactly, three
  or more by FABRIK from the animated pose, seeded on the `bend` side:
  position passes with no trigonometry inside them, as og-anim's 3D
  chains are solved), `transform` constraints (bones moved by a mix
  towards a target bone's world transform) and `along` constraints
  (bones placed on a `path` attachment, a cubic spline drawn as nothing,
  with DragonBones' position, spacing and rotate modes), and clips of bone (rotate, skew, translate, scale), slot
  (attachment, colour, offset), deform, draw-order (`order`) and IK
  (weight and bend) keys with events; keys ease linearly, hold (`step`)
  or follow a cubic Bézier (`curve`, solved by bisection), and a rotate
  key turns the shorter way or, with `clockwise n` or `counterclockwise
  n`, that way through n more whole turns. A child
  armature plays its parent's clip by name when it has one, else its own,
  at the parent's time. The cook stores the canonical text in an
  `og.Rig2D` blob, written by hand or imported from a DragonBones 5.5
  export with its atlas (`dragonbones "x_ske.json"`, `atlas
  "x_tex.json"`): skew, colour offsets, zOrder and IK timelines, armature
  displays (as child armatures), bounding boxes, path displays and path
  constraints, every skin (the `default` one the rig's own attachments,
  each other a `skin` block of swaps; a child armature's skins its own, worn by the name the rig wears), Bézier curves,
  `clockwise` turns (accumulated as DragonBones' parser does) and the
  sine `tweenEasing` (above 1: Hermite segments of a `curve`, within 1e-7
  of the sine) come across, and a frame without `tweenEasing` holds as DragonBones holds
  it. Or the cook rigs it from layered art (`parts "x.psd"`, or a
  folder `parts "x/"` of PNG parts, and a `template`; `cook/autorig.rs`):
  each layer named for a part of the template, the joints where the
  parts overlap near their joint ends, the template's bones through
  them, two-bone IK on the legs, the parts that bend (limbs, the torso,
  a cape's or a tail's chain) meshes weighted across their joints
  through a joint bone a transform constraint turns half-way (so a knee
  bends without pinching), the template's clip library (idle, walk, run,
  jump, attack, hit, die, wave, cheer, nod: hand-keyed at their poses,
  sampled into keys, scaled to the rig's leg length, with the head and
  torso counter-rotating and a cape's, a tail's or hair's chain lagging),
  the parts packed into an atlas cooked
  beside the rig as `<rig>#atlas`, so the setup pose draws the art as
  drawn; a layer whose pixels Photoshop would show otherwise (effects,
  blend modes, clipping, adjustments) or that fits no part is refused,
  naming the layer, the file and the fix. Spine's format is not imported (its licence); the module docs of
  `rig2d.rs` give the mapping a converter follows. `og.load.rig2d`
  registers a rig and its texture (`og.anim.rig2dInfo` reads it back;
  `og.anim.clip` finds a clip by name). `og.Skeleton2D` (`rig`, `clip`,
  `time`, `speed`, `loop`, `flipX`, `layer`, `skin`, `finished`) plays it through
  the `og.skeleton2d` system, run with the tweens and flipbooks after the
  game's systems; clip events arrive as `og.Skeleton2DEvent` (a child
  armature's tagged with its name, forwards and backwards);
  `og.IkTarget2D` moves a constraint's target (its weight multiplies the
  constraint's) and reads back `reached` and `error`;
  `og.anim.skeleton2dBoxes` answers the bounding boxes the last pose
  showed. The pose is a function of the component, its Transform and its
  IK targets, every angle through arithmetic-only sincos and atan2, so it
  is the same bits on both hosts; the playback position and the boxes are
  the state block. Extraction draws each skeleton as one
  `og_render::SpriteMesh`: the sprite batch extended to triangles
  (`sprite_mesh.rs`, a corner's colour offset added after its multiplier
  in `fs_mesh`), sorted with the sprites of its layer and lit by
  `og.Lighting2D` as a sprite there.

### 2.8 Platform SDK as host ABI **(D)**

The engine imports a small set of host functions, each a capability the shell
grants: `og_net_fetch` (allowlisted, game-scoped token), `og_store_*` (cloud
save), `og_leaderboard_*`, `og_room_*` (rooms/RTC via the shell), `og_telemetry`,
`og_point` (Origin Point), `og_ai_mark`. In TypeScript they appear under
`og.platform.*`. Nothing in game code can reach cookies, the DOM or arbitrary
origins; the shell owns every credential.

**Rooms.** `og.platform.roomCreate`, `roomJoin`, `roomLeave`, `roomSend`,
`roomPeers`, `roomInfo` and `roomMessage` put this engine with other
players' engines. The engine never opens a socket: it queues ops and its
host carries them. Three host ABI functions do the carrying:
`og_room_take` (the queued ops, JSON), `og_room_deliver` (one event in)
and `og_room_set_host` (`relay`, `player` or `none`). The wire shapes are
og-net's protocol (`crates/og-net/src/protocol.rs`). The ops are `create`,
`join`, `quick`, `leave` and `send`. The events are `joined`, `peer_joined`,
`peer_left`, `message`, `left` and `error`.

- **Quick play.** `og.platform.quickPlay { mode, players, timeoutMs }`
  (op `quick`) is matchmaking: the host queues the engine with others of
  the same game, mode and player count, and once `players` wait it puts
  them in a room of their own (nobody joins it by id), each hearing
  `joined`; a wait past `timeoutMs` ends in `error` `timeout`, a
  `leave` takes the engine out of the queue (og.RoomJoined `cancelled`).
  Natively `og relay` keeps the queue; on the web the platform's
  matchmaking does (platform ask 21).
- **Peers.** A room numbers its peers in join order from 0 and never
  reuses a number.
- **Ordering.** Messages from one sender arrive in the order it sent them,
  and its departure arrives after its last message. Lockstep's agreement
  on a leaver's last input rests on this.
- **Events in the tick.** Events arrive at the start of a tick, like host
  input, as og.RoomJoined, og.RoomPeerJoined, og.RoomPeerLeft and
  og.RoomMessage.
- **The engine's own channel.** A channel that starts with `og.` belongs
  to the engine (lockstep's `og.lockstep`), and its messages never reach
  the game.
- **Limits.** A game message is at most 64 KiB; an engine message is at
  most 16 MiB (a snapshot for a late joiner).

Each host carries rooms its own way:

- **Natively.** `og relay [--port P] [--host <ip>]` (og-net's relay; the
  default port is 47900) serves rooms by URL path. `og run --room
  ws://127.0.0.1:P/<room>` connects through og-net's client on a thread of
  its own and joins `<room>` before the first tick. The run pumps the link
  around every frame. `--latency MS --jitter MS --loss PCT --seed N` make
  the relay a bad network (every frame held, unreliable sends dropped and
  reordered), so netcode is tested against latency and loss locally.
- **On the web.** The Worker posts the engine's ops to the page and
  delivers the page's events. The shell's `RoomHost` maps them onto the
  Player's `room.*` (docs/platform-asks.md #16, open). A Player whose
  hello does not list `room.join` leaves the host as `none`, and
  roomCreate and roomJoin are refused. A Play link to a room
  (`?room=<id>`) is read by the play page and handed to the engine with
  the host (`og_room_set_host`'s `join`): the engine joins it itself, once,
  as soon as a host carries rooms, so the game hears the same
  og.RoomJoined as for its own roomJoin, and roomInfo's `link` names it.
  On a page with no room host og.RoomJoined says so (ok false). The
  Worker's `advance` and `frame` calls pump the room as its own frames do,
  and its `net` call answers og.net's status and shared hash. Two
  Worker-hosted pages joined by a Play link play the arena in lockstep
  through a relay (`packages/engine/tests/web-rooms.test.ts`).

**Purchases, ads, achievements, identity and share** (G3-6;
docs/platform-asks.md #19). `og.platform.products`, `purchase`,
`entitlements`, `consume`, `adReady`, `showAd`, `achievementUnlock`,
`achievementProgress`, `achievements`, `player`, `friends`, `inviteSend`,
`invites` and `share` are requests on the same channel. The host answers
them with game-scoped opaque ids. Accounts and money stay the platform's.
`og.platform.supports` asks whether the host answers a method; one it
lacks is `unavailable`.

- **Notices.** Unasked host input (a pending purchase settled, entitlements
  changed, an invite) comes in through `og_platform_notify` and is applied
  at a tick's start as `og.PlatformNotice`, logged with the answers.
- **Grant once.** `og.platform.grant` records a delivered purchase in
  `og.PlatformGrants`, which is world state, so a consumable is delivered
  once across restores, replays and relaunches. Entitlements are always
  the host's answer.
- **Ads.** The host pauses the game through its lifecycle while an ad
  shows and answers after the resume.
- **Sessions.** In a running lockstep or rollback session a system that
  writes shared state hears no platform answer or notice: they are this
  peer's alone.
- **Replay.** `og replay` keeps every answer and notice with its tick and
  feeds them back on compare.
- **The fake.** Natively and in playtests a deterministic fake answers
  (`og run --platform-fake`, `PlaytestDesc.platform`); a published page
  never fakes them.

### 2.9 The video producer

The engine already has what "video is code" pipelines bolt on (a fixed
tick, exact seeking by snapshot, headless runs, an offline mixer), so a
video is a command script played headless and recorded (docs/depth.md §3).

- **Frames.** `og render [project] --commands "..." --fps F --from S --to S
  --size WxH` runs the game on the native host at its own tick and draws
  the world once every `rate / F` ticks (F must divide the tick rate;
  otherwise the error names the nearest that do), so frame i is the world
  at tick `round(S · rate) + i · rate / F`, never between ticks. The ticks
  between frames are simulated without drawing. Each drawn frame is read
  back from the render target through og-probe's `Capture::frame` (the
  frame the loop drew, no second draw, no PNG, no JSON-RPC) and streamed
  to `og` as RGBA rows (`packages/engine/src/node/render.ts`), which
  writes PNG or JPEG frames, pipes them to ffmpeg, and hashes them.
- **Audio.** `--audio` runs the capture mixer (`og run --audio`'s) from
  tick 0 to the range's end into a 16-bit WAV, muxed into the video (AAC
  in an `.mp4`, Opus in a `.webm`); it is rendered from the tick's command
  stream, so it is the same bytes in every run.
- **Encoding.** ffmpeg is found through `OG_FFMPEG` or `PATH` and reported
  by `og doctor`; `.mp4` is H.264 (yuv420p, CRF 18) and `.webm` VP9 (CRF
  32). Without ffmpeg the render writes its frames and says so: nothing in
  the engine needs it.
- **Determinism as a contract.** `frames.json` lists every frame's tick,
  time and the SHA-256 of its RGBA bytes (hashed before any encoding) and
  the audio's SHA-256; two runs of one script give the same list (a test,
  `packages/engine/tests/og-render.test.ts`). A changed hash is a changed
  frame, which is how a review finds what a change moved.
- **Jobs.** `--jobs N` splits the range in N parts. A leader process
  simulates the script without drawing and writes a render snapshot
  (og-probe's `render_snapshot`: the world exactly, input resources and
  the probe's pointer included, unlike a save) eight frames before each
  part; each part restores its snapshot, drops the script steps the leader
  already ran, draws those eight frames unsent (the renderer's history,
  TAA and the adapted exposure, settles) and renders its frames; ffmpeg
  concatenates the parts. The frames equal a one-job render's (the same
  test).
- **The review surface.** `--contact-sheet COLS` writes a grid PNG of
  frames spaced over the range (COLS columns, ceil(COLS / 2) rows), each
  labelled with its time and tick in a built-in pixel font: an agent
  reads one image instead of watching a video, then opens the frames the
  sheet points at. `--timeline <name>` is `--commands
  "0:timeline:play:<name>"` with the timeline's `duration` as the range,
  once timelines exist (§2.7, docs/depth.md §2).
- **Footage.** A `type video` asset (`.ogt` naming an `.mp4`, `.webm` or
  `.mov`, with `fps`, `maxSeconds`, `size`, `kind`, `audio`) is cooked by
  ffmpeg (ffprobe plans it) into an `og.Video` blob, a texture array of
  its frames (`<id>#frames`, each a KTX2 layer of one mip level, at most
  256) and its sound (`<id>#audio`, QOA); the cook reports the bytes and
  warns past 64 MiB. `og.load.video` keeps the frames as the device
  encodes them and makes one Texture of a frame's size (og-render's
  `FrameTexture`) whose contents become the frame shown, so a material
  map, an `og.Sprite` or a UI image draws footage as any texture, at one
  frame's GPU memory. Each further player of a video gets a view of its
  own over the same decoded frames (`VideoPlayer.texture`), so players at
  different times show different frames. `og.VideoPlayer` (`video`, `playing`, `time`,
  `speed`, `loop`) is played by the `og.video` system by the tick (not the
  time scale) and keeps the entity's `og.AudioSource` in step: its
  `startOffset` is the video's time, and a seek stops it for a tick and
  starts it again there. A player's state is its component, so a restored
  snapshot plays on as the world that took it; at a 30 Hz tick, tick 90
  shows the source's frame 3 s in (`packages/engine/tests/og-video.test.ts`).
- **Streams (G3-2).** A clip stays for short loops; `stream true` cooks
  the source to AV1 (8-bit 4:2:0, BT.709 limited range; ffmpeg's
  libsvtav1, else libaom-av1; a key frame every `keyframe` seconds, one
  temporal unit per frame), read from the IVF ffmpeg writes into the
  engine's own `og.VideoStream` blob (`<id>#stream`: the sequence header,
  a frame table of offset, size and key, the units), so no container is
  parsed at run time. Beside it, `<id>#poster` (frame 0 as rav1d decodes
  it, an ordinary texture) and `<id>#audio` in Opus. The stream's file is
  not part of the startup bundle: `og.load.video` reads it (on the web,
  fetches it). Each player draws into a stream texture (og-render's
  `add_stream_texture`: a GPU texture of the frame's size written in
  place, so a new frame rebinds nothing). Natively each player has a
  rav1d thread (`crates/og-video`; rav1d is BSD-2-Clause, the
  memory-safe Rust port of dav1d) that decodes ahead of the player and
  from the key frame at or before a seek; the web decodes with WebCodecs
  in the engine Worker, or rav1d built for wasm in a Worker of its own
  where the browser has no AV1 decoder (`dist/av1/og-av1.wasm`, 1.18 MB,
  311 KB brotli, fetched only then and never part of the engine's wasm or
  a page's boot). On one thread it decodes, in Safari 27 on Apple silicon,
  720p at 46 fps on worst-case noisy footage (221 on clean) and 1080p at
  19 (94 clean), so a stream larger than 1280x720 (720x1280 portrait) also cooks a 720p
  rendition (`<id>#stream720`, `og_video::stream::fallback_size`) which
  the fallback plays instead, encoded 6 crf coarser than the stream
  (`crf720` overrides it): a 4 s 1080p test clip's stream is 786 KB, its
  rendition 408 KB (552 KB at the stream's crf). A player fetches only
  one of them, and a publish build's service worker precaches neither. `og.video.capabilities` reports which
  decoder runs (`rav1d-wasm`, hardware false, max 1280x720).
  **Time:** `VideoPlayer.time` is world state advanced by the
  tick (dt × speed), frame = floor(time × fps), exactly as for a clip;
  the sound is the AudioSource started at `startOffset` = time, so the
  tick is the one clock and picture and sound cannot drift. Three things
  keep that true over a long stream: `time` is an f64 summed with og.audio's
  tick length (1 / the whole tick rate), as the sound's playhead is (in
  f32 an hour at 60 Hz fell 2.8 s behind it); the cook pads the sound
  with silence to the picture's length; and the source's loop region ends
  at the picture's end (`loopEnd` = frames / fps), so both wrap on the same
  sample. og.VideoPlayer is at version 2 for the f64 `time`
  (`save_migrations` widens a version 1 save's f32) and lazy: registered
  when a game first names it, so a world without footage has neither the
  type nor a hash its changes move. Pausing is the player's `playing` (the sound stops, and resumes
  at the frame it held) or the game's ticks stopping (a hidden page, the
  shell's pause), which stops both; the time scale and hit stop leave
  footage alone, as they leave its sound. Several players of one stream
  each have a decoder of their own (natively a rav1d thread; on the web a
  WebCodecs decoder, or a rav1d decoder in the one fallback Worker, which
  takes the players' units in turn), so one never decodes for another. Decoding is
  presentation: what a decoder has ready never reaches the world. An
  executor (`og run`, `og render`, `og test`) waits for each player's
  exact frame before drawing, so captures are deterministic; a game in a
  browser shows the newest frame decoded and keeps it while the next is
  late.
- **The CPU-only sandbox's cost.** On lavapipe (16 cores) the 3D template
  renders a 1280×720 frame in about 0.39 s in one job and 0.15 s a frame
  with `--jobs 4`; at 640×360 about 0.11 s. An 8 s clip at 30 fps is
  240 frames: about 90 s at full size in one job, 35 s in four. Agents
  iterate at 640×360 and review the contact sheet, rendering full size
  once. `og render` and its capture members are development members:
  absent from `publish` builds.

### 2.10 Lockstep

Deterministic play over a room (IDL module `net`, crate `og-net`). Every
peer runs the whole world; only the players' inputs and the world's hash
cross the room. Lockstep rests on §1's determinism and snapshot.

- **Settings.** `og.Lockstep` holds `enabled`, `players`, `inputDelay` and
  `hashEvery`, which the game sets. The engine sets `localPlayer` and
  `running`. The resource belongs to this peer's session: it is never
  hashed or compared.
- **Inputs.** `og.PlayerInput` is a reflected component: `player` (the
  seat), `buttons`, `x`, `y`, `aimX`, `aimY` and `tick`. One entity per
  player carries it. At the start of each tick the engine writes every
  field except `player` from that seat's input for the tick, and zero for
  a seat with no input. The input's packed bytes, read through the
  component's layout, are what crosses the room. A game hands over its
  own input with `og.net.submit`, called from the only system that reads
  `og.Input`; it acts `inputDelay` ticks later. Every other system reads
  `og.PlayerInput`. Without a room, or with lockstep off, a submit drives
  the local player on the next tick, so the same game plays alone.
- **Input shapes and commands.** `og.net.shape` (outside a system, before
  the room starts, the same on every peer) replaces `og.PlayerInput` with
  a game's own component (a u32 `player`; a u32 `tick` is written when
  present; `og.net.submitInput` gives it) and declares a command record:
  orders that act on one tick only (`og.net.command`, at most
  `maxCommands` per seat and tick). A seat's input for a tick is its
  state's packed bytes followed by that tick's commands, packed one after
  another; `og.net.commands` reads them at the tick on every peer, by seat
  then in the order given, with `player` set to the giver's seat. A
  prediction repeats a seat's last state and never its commands, so a
  rollback never acts on an order twice; a command given while a tick is
  re-simulated is dropped (its first run sent it). The start message
  carries the shape's fingerprint (its records' names and field types),
  and a peer whose shape differs refuses it. Lockstep's messages are at
  wire version 4 (2 for rollback, 3 for the seat-change epoch, 4 for
  commands and the shape).
- **Start.** Until the room holds `players` peers, each world runs freely
  (the lobby). Then the authority starts lockstep: the authority is the
  lowest peer number, and its settings become the room's. Every other
  peer restores the authority's snapshot, and the seats go to the peers
  in number order. Once the room is full, every other peer holds its
  world until that snapshot arrives (for at most `stallTimeout`, 5 s when
  it is 0): its lobby ticks would be replaced anyway, and a slow
  authority must not find a scripted peer's steps, its `quit` among them,
  spent in a lobby. Lockstep messages that arrive after a `start` and
  before it is applied (a `drop`, an `admit`, inputs) act once its run
  began, in order.
- **The tick gate.** Tick t runs only when every seated player's input for
  t is in. The gate is og-probe's (`set_gate`, consulted by step, frame
  and advance), so the tick itself is unchanged and a frame that cannot
  run a tick simply runs none.
- **Rollback.** With `og.Lockstep.mode` `rollback` (the authority's is the
  room's), lockstep is the case of prediction depth 0: a tick whose inputs
  are not all in runs on each missing player's last input, up to
  `maxPrediction` ticks (and no more than the peer can re-simulate in half
  a frame) past the last confirmed tick. The engine keeps a snapshot of
  every tick not yet final in a ring of its own; when a real input
  differs from its prediction, the gate rewinds the world to that tick's
  snapshot (`save::rewind`, crate-internal: ring bytes only, never a
  game's save, so no check, migration or `og.WorldRestored`) and runs the
  ticks again in the same frame. A tick is final once it ran on real
  inputs only; the hashes the peers compare, and `og.net.finalHash`, cover
  final ticks alone, so the confirmed world equals lockstep's for the
  same inputs. Presentation is deduplicated outside the tick: a
  re-simulated tick replays each sound once (a new play is kept only if
  at most 100 ms of wall time late), the view snaps, and
  `og.net.resimulating` tells a game's own effects to hold.
- **Stall, drop and rejoin.** A running peer silent for a second is
  reported (`og.PeerStalled`, session-only, and the status's `stalled`).
  Silent past `og.Lockstep.stallTimeout` (5000 ms; 0 never drops), or gone
  from the room, its seat is dropped: the acting authority (the lowest
  running peer not itself silent) asks every other running peer how far
  it holds the seat's inputs, each stops taking more, and the seat empties
  from the tick after the furthest, with the inputs up to it handed to
  all. So `og.SeatDropped { seat, peer, tick, left }` arrives at one tick
  on every peer (in rollback, a peer that predicted past it re-simulates),
  and the seat's input is empty from then on: a game hands it to a bot if
  it wants one. Wall time decides only when a peer is reported and
  dropped. A dropped peer that comes back holds its world and asks again;
  with `rejoin` (the default) the authority gives it back its seat (or a
  newcomer of the same name gets it) from a snapshot, from a tick past
  every input the authority has sent, and `og.SeatRejoined` arrives at
  one tick everywhere. Both seat events are hashed and lazily registered
  (only a world that runs lockstep in a room has them). Each input
  message names the seat changes its sender knows, so one that overtook
  an `admit` or a `drop` waits for the resend.
- **Late join.** A peer that joins a running room asks for a snapshot. A
  running peer sends one, and the new peer takes a free seat (or watches)
  from the tick named in the `admit` message.
- **Hashes.** Every tick, each peer hashes its world without its own
  session: the input resources, `og.Lockstep`, input, room, platform,
  asset and stream events, and the UI's focus. This is `og.net.hash` and
  `og run --json`'s `netHash`. Every `hashEvery` ticks a peer sends the
  hashes since its last batch, so the first divergent tick is exact. On a
  mismatch, `og.Desync { tick, peer, peers }` is sent once, and both peers
  exchange their snapshots of that tick.
- **The desync dump.** The probe's `desync` dump (`dump_desync`) names the
  tick, both hashes and the component-level diff: per entity (by name),
  component and field, the local and remote values, plus resources,
  events, blocks and entities present on one side only. A system that
  reads `Math.random()` shows up as exactly its field
  (`packages/engine/tests/rooms.test.ts`).
- **The proof.** `templates/archetypes/arena` (Coin Duel) is two players in
  lockstep. Its playtest `tests/duel.og.ts` (`PlaytestDesc.room`) runs two
  native processes through a relay that the playtest starts; both quit on
  tick 600 and hash the same there. A script's steps sit on the room's
  shared ticks once lockstep runs (the gate's snapshot restore leaves the
  schedule on them, unlike a game's own restore, which moves it), so a
  `quit` or an `expect` lands on one tick for every peer, and a `then`
  step relaunches every place into a room of its own.

### 2.11 The model review

A model built as code from a reference image (the og-model skill, after
img2threejs) is judged pass by pass on one sheet. The engine does the
mechanical half of every judgment, so an agent spends its attention on
the sheet and nothing else.

- **Fixed cameras.** The command channel's `view:<entity>:front|back|side|top`
  frames the entity's drawn bounds (its og.Parent children with it)
  orthographically from +Z, -Z, +X and +Y (the top with the front at the
  bottom), and `view:<entity>:orbit:<degrees>` in a 30° perspective from
  20° above the horizon, a tenth of the frame free on each side, until
  `view:off`. `turntable:<entity>:<n>` queues n orbit views 360/n apart
  with a screenshot each for the next tick's start, then the view before
  (`crates/og-runtime/src/review.rs`). The view is render state: it
  replaces the camera extraction made from og.Camera, which is never
  written, so no snapshot, replay or hash sees it. The bounds come from
  the last frame's instances of the entity before the models pick their
  levels of detail (the level drawn is the one the review camera sees) and
  again from the frame's own; a skinned model's posed parts are not
  framed. Not built into publish.
- **The numbers and the sheet.** `og compare <reference> <render>`
  (`crates/og-cli/src/compare.rs`) keys each silhouette (alpha where an
  image has it, else ΔE00 above `--key` from the median border colour),
  prints the silhouette IoU and the CIEDE2000 difference (sRGB, D65) over
  the pixels both cover, mean and 95th percentile, per region of a flat
  colour mask or over the overlap, and writes one PNG: the reference, the
  render as compared, the silhouettes overlaid, labelled. The render is
  compared in the reference's frame (resampled when the shapes agree,
  refused otherwise; `--align` matches the silhouette boxes). No clock,
  no randomness: two runs give the same numbers and the same bytes.

## 3. API surface

### 3.1 One IDL, two hosts **(D)**

The engine's public API is declared once (`crates/og-api/api/<module>.ogt`, the same
text format): modules, functions, desc structs, enums, events, with docs and
ranges. From it the build generates:

- Rust trait impls checked at compile time (`og-host-web`, `og-host-node`),
- the TypeScript `.d.ts` and the runtime binding tables,
- the wiki pages (`packages/knowledge/wiki/`), one page per module, with
  the signatures quoted from the IDL so they cannot drift,
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
parameter names and `og.load.gltf()` follow three.js naming so an author's
prior transfers, while the ECS, the tick and determinism are the engine's.

**The math layer** (depth S9): three.js's methods as pure functions over
the plain structs, declared in the IDL's `math` module (`host false`: no
binding symbol) and implemented in TypeScript (`packages/engine/src/math.ts`),
exported as `vec3`, `vec2`, `quat`, `mat4`, `box3`, `sphere`, `plane`,
`frustum`, `ray` and `color`: `vec3.add, sub, scale, dot, cross, length,
normalize, lerp, distance, applyQuaternion` (and `vec2` likewise, with
`rotateAround`), `quat.multiply, slerp, fromEuler, invert, rotate`,
`mat4.multiply, invert, transformPoint, transformDirection, compose`,
`Box3`, `Sphere`, `Plane`, `Frustum` (`containsPoint`, `intersectsBox`,
from the camera), `Ray` (`intersectBox`, `intersectSphere`, `intersectPlane`,
`intersectTriangle`, null on a miss) and `color.fromHex, toHex, lerp,
fromHsl` (hex and HSL are sRGB, the result linear, as three.js's colour
management). No classes and no mutation of arguments: values stay plain, so
columns and reflection are untouched and a result can be written straight
into a column. Only `+ - * /` and `sqrt` (sincos as the camera systems'),
so a system using them stays deterministic. `packages/engine/tests/math.test.ts`
holds a table of cases against three.js's documented results.

### 3.3 Command channel **(D)**

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
| `dump_memory[:path]` | the probe's memory report: resident (native) or linear (web) memory, the counted heap, entities, the stream budget, the audio bank, GPU memory by textures, meshes, render targets and buffers |
| `set:<entity>.<component>.<field>=<value>` | a Tune write through the schema |
| `seed:<n>` / `pause` / `resume` / `advance:<ticks>` | determinism controls |
| `describe[:path]` | the frame as data (§3.4): visible entities with screen boxes, depth order and occlusion; UI elements with rects and text; the camera |
| `expect:<operation>` | an assertion at this tick (§3.4); a failure fails the run |
| `save_scene[:path][:spawned]` | every scene's `.ogt` rewritten from the world, resources included; with `spawned`, the run-time entities under a marked section (§3.4) |
| `quit` | exit |

### 3.4 Probe **(D)**

The engine implements `RuntimeProbe` protocol 5 natively (`hello`, `tick`,
`state`, `advance`, `pause`, `resume`, `input`, `accepted`, `pick`, `entity`,
`bounds`, `patch`, `run`) plus the wire-only members `capture`, `sweep`,
`reload`, `snapshot`, `restore`, `dump`, `commands`. Transport: in-page calls
from the shell (no postMessage hop: the shell calls the Worker through a
request ring), a loopback WebSocket from `og` on native. `publish` builds keep
the player-facing members (`hello`, `tick`, `state`, `pick`, `entity`,
`input`, `capture`) and drop the rest, compiled out with what only they
read (`describe` and the views' id passes, `save_scene`'s scene writer,
`reload`, the dumps the modules register). Their command channel keeps the
actions og.ai's commands and a timeline's event keys reach (input, `set`,
`teleport`, the clock, registered commands) and refuses the ones that only
observe the world for a tool (`screenshot`, `describe`, `expect`, `get`,
`dump_*`, `help`, `listen`, `save_scene`) with a warning
(crates/og-probe/README.md §Builds).

Above capture sits the layer that lets an agent check a frame without
reading pixels:

- **The semantic frame.** `describe` (a probe member and the
  `describe[:path]` action) answers the current tick's frame as JSON,
  deterministic for the same world: the visible entities nearest first, each
  with its name, archetype, label, screen-space box and pick point (CSS
  pixels), depth and the fraction of it that others hide (objects in
  front, and UI elements drawn with an opaque background over it: the id
  pass's pixels under them show the UI); the UI elements
  with their rects and text (from `dump_ui`); the camera. The entities come
  from an entity-id pass: og-render's `id_pass.rs` renders the extracted
  frame (meshes, skinned meshes posed on the CPU, world sprites) at half
  the width and height, a quarter of the pixels, on the CPU, so a host
  without a GPU describes the same frame. `pick` answers from the same pass
  when the game has no picker, so a box `describe` gives contains the point
  `pick` names.
- **What the secondary cameras show.** The id pass runs again for each
  og.CameraView and og.Mirror, from its eye and on its layers, at a
  quarter of its image's pixels. `describe` lists the views (`views`: id,
  kind, image size and its entities, boxes in the image's pixels), and
  every main-camera entity names the views that also show it (its
  `views`). An entity on a layer only a view admits is invisible to the
  main pass and `pick`, and is found under its view: describe reports
  visibility per camera, never a merge. `expect:entity:...:within` and
  `pick` read the main camera.
- **Expectations.** `expect` operations run as actions (`expect:<op>`) and
  as a probe program step, and are evaluated at their tick:
  `entity:<id>:within:x,y,w,h` (its described box inside a screen box) or
  `within:x0,y0,z0,x1,y1,z1` (its position inside a world box),
  `event:<Name>[:count]`, `ui:text:[=]<text>`,
  `region:x,y,w,h:color:#rrggbb[:tol]`, `frame:<ref.png>:rmse:<=v` and
  `state:<path><op><value>` over the game's `state()` facts,
  `entity:<id>:<path><op><value>` over a field of the entity's
  `dump_entity` (`render.lod==2`) and `timings:<path><op><value>` over
  `state().timings` (`triangles<=1500000`). Each result
  keeps its tick, what was expected and what was observed; a failure is a
  warning in the log. `og run --json` carries them (`expectations`) and
  exits 1 on any failure; a playtest may be expectations alone. A frame
  expectation without a renderer is unchecked, never a failure.
- **Trace.** `og run --trace out.json` writes the run in the Chrome trace
  format (Perfetto opens it; `packages/engine/src/trace.ts` builds it on
  both hosts and checks it); every span carries the start it was timed
  at, so it lies where it ran. Threads: engine (each tick, its systems
  inside it from the world's clock), render (each frame, its render-graph
  passes' CPU encoding), gpu (the same passes from timestamp queries,
  where the device has them and the build is not `publish`, whose
  og-render leaves the queries out, `RenderCapabilities.gpuTimestamps`: each
  frame anchored at its submit, after the frame before it, since the
  GPU's clock is its own), load (each cooked output read), cook (the
  cook) and `cook N` (each output on the cook thread that made it), and
  `lane N` (each job og-lanes ran on pool thread N, with its site). The
  live probe's `state().timings` carries the last tick's `systems[]`, the
  last frame's `passes[]` and the GPU's `gpuPasses[]` of the newest frame
  read back (name, ms). On the web the engine Worker records the same
  document (the play page's `?trace` from the boot, or the probe's
  `trace("start")`) and the shell reads it (`trace()`, `saveTrace()`
  downloads it).
- **Scene write-back.** `save_scene` (member, and
  `save_scene[:path][:spawned]`) rewrites every scene the world was
  assembled from, each to its own `.ogt` (the boot scene's, then each
  `og.world.instantiate` made), line by line: a field line changes only
  when its value, rounded as the cook rounds it, differs; a field `set` or
  `patch` tuned that the text lacks gets a line (or a component block);
  comments, order and spelling stay. Resources are scene text too
  (`resource <Name> { field value }` at the top level sets those fields
  when the scene is instantiated, the others keep the world's values): a
  scene's resource lines are rewritten, and a tuned resource no scene sets
  gets a block in the boot scene. With `spawned`, the entities the game
  spawned at run time are written at the end of the boot scene under the
  line `# ── spawned at run time ──`, regenerated whole on every save,
  nested by og.Parent, each with every component whose fields scene text
  can give (a model's `model` line; a component holding a geometry,
  material, sound or entity handle is named in a comment, not written; an
  instance's root, which the instantiating code makes again, is left
  out). Cooking the result loads the same world, and saving again changes
  nothing (`packages/engine/tests/scene-write-back.test.ts`).
- **Diagnostics as data.** `og cook --report` writes
  `og-out/cook-report.json`. For each asset it records triangles,
  vertices and LODs, textures with sizes and formats, bones, clips, bytes
  cooked and budget flags, and every other output type its own figures,
  read back through the runtime's loaders: a sound its seconds, channels,
  rate and chunks; a font its glyphs, kerning pairs and atlas pages; a
  scene its entities, instances and components; a navmesh its tiles,
  polygons and vertices; a timeline its tracks and seconds; a string
  table its locale and keys; a style, behaviour, tileset, tilemap, rig,
  flipbook, shader, post pass, emitter, look, palette, LUT, environment
  map, video, terrain and stream pack theirs (a generated terrain its
  height histogram, land above the sea and river length, with its height
  and control maps as PNGs beside the report). `og doctor --project <dir>
  [--json]` checks it against the budgets project.ogt sets (`budgets {
  triangles vertices textureSize bones clips assetBytes totalBytes
  soundSeconds videoSeconds fontGlyphs navPolygons sceneEntities
  emitterParticles }`, each with a default) and prints findings with
  severity, asset, budget, limit, value and fix. One more key,
  `bvhShare` (30), is a warning, not a limit: a model whose cooked
  raycast BVH is over that percent of its blob and at least 16 KiB is
  named by `og cook --report` and `og doctor --project`, with
  `model { raycast false }` as the fix (the 30 % read from the
  repository's models, tests/evidence/debt2/size/NOTES.md).
  Source material warnings also name full-strength `[1, 1, 1]` emissive
  factors with an emissive texture whose mean linear Rec.709 luminance is
  >0.01 (sRGB RGB decoded to linear light; alpha ignored, as glTF emission
  ignores it). `emissiveTexture` warnings name the asset, material and mean
  so an author can check for albedo accidentally baked into emission. They
  persist on cached cooks and never modify the texture or material.

### 3.5 Hot reload **(D)**

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
snapshot make the restart lose nothing, where swapping code inside the
process would need an unloadable addon and wgpu's global state torn down.
A failed build keeps the old game; a change the addon does not link (a
test, another crate) restarts nothing. Measured 2026-10-26 in an orb on
`templates/game`, an og-runtime edit: 6.8–7.1 s from save to the restored
tick, of which the build 6.5–6.7 s, the handoff 51–61 ms and the new
process 254–322 ms.

### 3.6 The inspector and gizmos

A development build carries an inspector for humans (og-runtime's
`inspector` module, compiled out of `publish`): F1 shows it (Ctrl+F1 when
the game binds F1; `inspect:key:<code>` sets another), over every layer of
the game's frame, in the engine's default style. It reads only the
reflected schema and writes only through the probe:

- **Panel.** Entities by og.Parent and the world's resources, filtered by
  name; the selected one's fields, each with the editor its type and
  column kind ask for. Every edit is the probe's `set` (`Probe::tune`):
  checked against the schema, applied between ticks, kept for undo (a
  drag folded into one step), marked unsaved, and flagged when the game
  writes the value back. Save is `save_scene`, which writes each tuned
  field into its line of the scene text; a resource's field into its
  `resource` block.
- **Prefab overrides.** An entity placed from a prefab shows its prefab and
  node, and what its instance overrides: the scene's override lines and
  the session's unsaved edits, the overridden fields' names tinted. Each
  override, or all of them, is applied into the prefab's file
  (`apply_prefab`: every instance takes it) or reverted to the prefab's
  value (`revert_prefab`); either clears the fields' unsaved marks.
- **Where Save goes.** Natively the engine writes the files. A web page
  writes none: the save answers the texts, the frame answer carries them
  (`saved: {scenes: [{source, text}]}`), and the shell sends them to a
  Stage whose hello lists `scene_saved` as that event on the og-probe wire
  (at most 64 scenes an event; a scene outside `assets/`, not `.ogt` or
  past 1 MiB is not sent) and downloads each one otherwise
  (`shell/wire.ts` `routeSavedScenes`; `docs/platform-asks.md` #22). A
  prefab's Apply and Revert read the prefab's source file, so they work
  only where the engine reads the project's files.
- **Gizmos.** On the selected entity's og.Transform: translate (an arrow per
  axis, a square per plane, the centre in the view's plane), rotate (a
  ring per axis and one facing the view) and scale (a bar per own axis,
  the centre uniformly). W, E and R pick the mode (over the panel, or
  anywhere the game binds none of them), translate and rotate follow the
  world's or the entity's axes, and the handles keep one length on screen
  (96 px). A drag casts the pointer's ray from og.Camera against the
  handle's line or plane (a ring seen edge on turns by travel along its
  tangent), converts the world pose to the parent's space, and writes one
  field a frame through the same `set`, folded into one undo step; Ctrl
  snaps a move to 0.5, a turn to 15° and a scale to 0.1 of its factor.
  The selected meshes' bounds box draws with it.
- **Observable.** `dump_inspector` answers the panel's elements with their
  rectangles and the gizmo's mode, space, hovered and dragged handle and
  each handle's grab point on the surface, so a script presses what a
  person would; `inspect:mode:<m>` and `inspect:space:<s>` set the gizmo.

## 4. The play surface without iframes **(D)**

The platform already serves a top-level Player on the play origin for phones
(`play.<domain>/g/:slug`, `goals/play-surface` D1, D2). This design makes it
the only mode:

- Desktop and the creator Stage move to the same top-level page. The Portal
  opens play in the same tab with the one-time identity handoff already built;
  the creator overlay (chat, Stage controls, Tune, annotations) renders in the
  play page itself, talking to Agent Server over its thread socket with the
  scoped delegation the handoff mints.
- The play origin carries no Portal cookie; the shell holds a game-scoped
  player token. Game JavaScript runs in the page, and that is acceptable
  because the page has nothing beyond the game's own scope to steal; the
  engine's host ABI is the only route to platform APIs. This matches the
  existing rule that games never receive Portal cookies, without the frame.
- COOP/COEP on the play page enables lanes; the per-publication declaration
  mechanism stays.
- The shell is the brand surface: boot splash (`genesis` motion), real
  progress (`progress`), ready-to-interact gate, safe areas, `100dvh`, one
  audio unlock, PWA install.

## 5. Executors and the sandbox **(D)**

| Class | Contents | Image estimate |
|---|---|---|
| `engine` | bun, `@origingame/engine` (wasm + native addon), git | ≈ 400 MB |
| `probe` | the same plus ffmpeg (minimal static build) | ≈ 600 MB |
| `asset` | plus Blender headless (trimmed) and the cook tools | ≈ 1.8 GB |
| `browser-check` (evaluation only) | plus Chrome headless-shell for the final "runs in a real browser" check | ≈ 800 MB |

No Godot, no Chromium in the inner loop, no Rust toolchain anywhere in the
sandbox. An image installs the published packages at the version the
platform pins (`bun install` of `@origingame/engine`, which carries the knowledge, and
`@origingame/create-game`, with their `optionalDependencies`: the platform
addon and `og`, tsrs), the same install an author's machine and the
platform's local stack make; no layout built from a git revision, no
engine checkout (`docs/release.md` §1, 2026-10-22). Sweeps, replays,
determinism and the probe's dumps run through
`og` on the native addon without a GPU and are deterministic. Frames need
a renderer: the `engine` and `probe` classes carry none (the CPU-only
renderer on Mesa lavapipe, which they carried through the build-out, was
set aside on 2026-10-22 to put features first; its stages stay in
`scripts/wave-end.sh` for when it returns), so a playtest's screenshot
checks there report "no renderer, frame unchecked" and pass, and a frame is
verified on the author's machine, in `browser-check`, or on an executor
with a GPU. Performance truth comes from the person's
device through Player telemetry and from `og run --device` (safaridriver /
Chrome DevTools on a paired phone) when the person is present.

## 6. Failure modes

| Violation | Behaviour |
|---|---|
| stale typed view used after a structural change | throws with the owning tick and the system name |
| dead handle | error value `{ok:false, reason}`; never a trap |
| system writes a component it did not declare | refused at registration with the field name |
| wall-clock read inside a tick | not reachable: `Date`/`performance` are not exposed to systems; the lint flags imports |
| wasm trap (engine bug) | the shell shows the crash card with the last 200 log lines and the world dump; telemetry uploads the report |
| WebGPU unavailable | GL fallback with the tier reported in `hello`; the agent reads the tier in Progress |
| cook stamp mismatch at load | refuse with the asset path and the two stamps |
| probe refusal | `false`/`null` exactly as protocol 5; `error` only for malformed calls |

## 7. Budgets

Measured in the spike (§10) and gated in CI afterwards. Quality comes
first (the owner, 2026-11-01): the engine targets current devices only
(§8), and no budget here is a reason to cut a feature to a reduced tier
or to hold a model or an effect below the quality it needs; a figure a
feature moves is measured at a wave's end and pursued, or the budget is
raised in its own commit.

| Budget | Target |
|---|---|
| engine wasm, brotli | ≤ 16 MB per download (the largest variant a player loads), raised from 4 MB on 2026-11-01: a ceiling against accidental growth, not a figure to cut features for; size is still pursued at wave ends. Model weights and other assets are not in it |
| first frame, a current iPhone, 4G, small game | ≤ 2.0 s click to first frame (the play-surface budget) |
| first frame, iPhone 13, Wi-Fi, mid-size 3D scene | ≤ 1.5 s |
| TS system over 10,000 entities (one archetype, 2 components) | ≤ 1.0 ms per tick on a current iPhone |
| complex scene (`tools/bench/complex`: a 320 m textured level, 5,261 props from 19 models in 60 materials, 48 skinned characters on 14 clips, the sun's cascades and 4 shadowed local lights, 20 particle emitters, transparency, AO, bloom, ACES, the tier's antialiasing) | ≥ 1.5× the frame rate of the same scene in Three.js on the same device, judged on WebGPU |
| reference scene (500 instanced objects, 2 shadowed lights, 20 skinned characters, PBR, post) | reported beside the complex scene; no longer the judge |
| probe `capture` 1280×720 | ≤ 30 ms native, ≤ 60 ms web |
| hot reload | ≤ 300 ms from bundle write to first new frame |
| headless executor image (`engine`) | ≤ 600 MB |

The Three.js budget is judged on the complex scene since the owner's
decision of 2026-10-25: a small scene hides the gap between the engines
(the reference scene measured 1.93× on WebGPU and 1.27× on WebGL2 on an M4
Pro), so the judge is a level at the size of a shipped game, built from
the same glTF models and cooked textures on both sides by
`tools/bench/threejs-compare.ts`; `tools/bench/README.md` names every
feature on each side and how it is matched.

The hot-reload budget is rebuild and swap. `og dev`'s type check of the same
save runs beside the rebuild in a process of its own and is reported apart
(`check.ms`, 33–48 ms on the template, `tests/evidence/tsrs/wave-2.md`); it
never delays or refuses a reload, so it is outside the 300 ms.

The wasm budget is per download: a page downloads one engine module, the
publish variant of its game's physics profile (3D or 2D) or, on a
cross-origin isolated page, the lanes variant, so each module is held to
it and the figure that counts is the largest one a player downloads, never
the sum of the modules a build ships (docs/web-export.md §Variants,
docs/ci.md §Updating a budget). The 2D profile's modules leave the 3D
world out as well as rapier3d (og-render's and og-runtime's `profile2d`:
terrain, planar water, the 3D fluids, scattered copies and volumetric
fog; `RenderCapabilities.world3d` is false, and `og.terrain`'s calls and
a `SHAPE_HEIGHTFIELD` collider answer a refusal naming it), so `og build
--publish` refuses a 2D project whose scenes hold them, by scene file and
line.

## 8. Platform matrix

| | WebGPU (Chrome, Edge, Safari 26+, recent Firefox) | native Linux (executor) | native macOS (developer) |
|---|---|---|---|
| renderer | full | full with a Vulkan device (a GPU; lavapipe set aside), else none: the simulation, the probe and the playtests without their frames | full (Metal) |
| lanes | with COOP/COEP | rayon | rayon |
| audio | AudioWorklet | cpal or null | cpal |
| capture | readback | readback | readback |
| determinism | yes | yes | yes |

**The compatibility baseline (the owner, 2026-10-27).** During
development features come first, and the engine targets current
hardware and current browsers; old browsers, low-end GPUs and old
drivers are not a target and are not tested. **Quality over reach (the
owner, 2026-11-01):** there is no reduced tier for weaker devices. A
browser or device without WebGPU is not supported (the player shell says
so instead of starting a degraded engine), and a feature is built for
the supported devices at the quality it needs, never cut down to fit a
weaker one.

- Browsers: the current stable Chrome and Edge and the major before it,
  Safari 26+, the current stable Firefox, on WebGPU. There is no WebGL2
  fallback (retired 2026-11-01, §12.4).
- Desktop: Windows 10/11 x64 on DX12 (in the browser, D3D12 for WebGPU)
  with a current NVIDIA, AMD or Intel GPU and driver; macOS on Apple
  silicon (Metal); Linux x64 with a Vulkan GPU.
- Phones: a current iPhone in Safari. Mid-range and older Android
  devices are deferred.
- The reference GPUs are real ones: `metal` (macrunner) and DX12 with the
  Windows browsers (studio-win). lavapipe is only the renderer of
  machines without a GPU (orbs, CI): a smoke check that a frame draws, and
  never a reference tier.

Windows x64 uses the same native API: the MSVC-built
`@origingame/engine-win32-x64` carries `og.exe` and `origin-engine.node`.
Its automatic renderer is DX12, lanes use rayon, and capture uses GPU
readback. `og check` reports the pinned tsgo fallback because tsrs has no
Windows package. Host validation is recorded in
`tests/evidence/release/windows-x64.md` (release row F7).

## 9. Branding **(D unless noted)**

- **Name**: Origin Engine. The platform's current `engine/` workspace (the
  Godot distribution) is renamed in documentation to "the Godot distribution"
  during migration and retired after it; the name belongs to this engine.
- **Prefix**: `og` everywhere a short identifier is needed: the CLI (`og`),
  crates (`og-*`), formats (`.ogt`, `.ogb`), the host ABI (`og_*`), the
  TypeScript root object (`og`). The platform already uses `og-` for
  credentials, `og-probe` for the wire and `OG_*` for environment, so the
  family stays consistent.
- **Packages**: `@origingame/engine` (the knowledge inside it as
  `knowledge/`), `@origingame/engine-darwin-arm64`,
  `@origingame/engine-linux-x64`, `@origingame/create-game`. Repository
  `originai-game/origin-engine` (`docs/release.md` §2).
- **Artifacts**: `origin-engine.wasm`, `origin-engine.node`; telemetry and
  user-agent token `OriginEngine/<version>`; `hello.engine` reports
  `origin-engine <version>`.
- **Marks**: the boot splash is the master symbol in white on ink `#14161b`
  playing `genesis` once; loading uses `progress` with a real value; the
  "Made with Origin Engine" badge in the player shell and the exported page's
  footer. A product mark `engine` is added to `packages/brand` following the
  product-mark rule (compositions of the master's parts): the core ring with
  the symbol's two **inner** arms — the engine is what turns inside the
  shell, the counterpart of `kit`, which is the two outer arms around the
  core. Geometry lives in
  `packages/ui-svelte/src/lib/brand/product-marks.ts`, never redrawn
  elsewhere.
- **Game UI is not brand, and brand is not game UI**: a game's UI style
  (og-ui, a look's `ui` line) styles that game's own screens, and nothing of
  the platform's design language reaches them: no token, no radius, no
  palette. Only the engine's marks, the boot splash, the loading state and
  the player shell (Back, "Made with") stay the platform ink and tokens.
- **Voice**: documentation, the wiki and `og` output use the platform's
  editorial voice (`origingame/goals/editorial-design`): plain sentences, no
  exclamation marks, every error names the fix.
- **Versioning**: semver; a Game pins its engine version in `project.ogt`;
  upgrades are explicit threads, as D19 already says.

## 10. Spike (go/no-go)

Two to three weeks, one branch, no platform changes:

1. Rust + wgpu renders a glTF PBR scene with one shadowed directional light
   and IBL in a Worker on an OffscreenCanvas, on a page served from
   `play.origingame.ai` with COOP/COEP, on Chrome, Safari 26 (iPhone) and the
   WebGL2 fallback.
2. The same crate builds as a napi addon and renders the same frame headless
   on Linux lavapipe inside today's `probe` sandbox; the two PNGs are compared.
3. A TypeScript system over 10,000 entities through typed views, measured on
   a mid-range Android.
4. The reference scene in Three.js and in the spike, same device, frame times
   recorded by the same harness.

Numbers against §7 decide: all four within budget → phase 1; wasm size or
iPhone first frame off by more than 50 % → redesign the renderer tiering
before phase 1; the TS view budget missed → move more systems into the
engine as data-driven behaviours and re-measure; the Three.js comparison
below 1.2× → the performance claim is dropped from positioning and the
project continues on the AI-native and isolation grounds only.

### 10.1 Results so far (2026-10-17, `docs/spike-results.md`)

Taken on an M4 Pro with headless Chrome and in the platform's E2B `probe`
sandbox; the iPhone and Android numbers are still pending.

- Size: the renderer-only wasm is 0.91 MB brotli (3.19 MB raw), against the
  4 MB core budget.
- Sandbox: the same binary renders the same frame on lavapipe (RMSE 0.003
  against Metal) at 236 ms per 880k-triangle frame, with no browser or Godot.
- Browser: GPU-bound instanced scenes are parity with three.js (within
  ±20 %, three's WebGPU renderer slightly ahead); per-object scenes of
  20,000 moving objects run 7.5× faster than three.js written the way
  authors write it (5 draws versus 20,001). The WebGL2 fallback costs 30–45 %.
- TypeScript systems over typed views: 0.014 ms per tick for 10,000
  entities on desktop, 60× under budget.

Consequence for the positioning (§1 goal 4): the engine's performance claim
is "batched, instanced and culled by default, so the slow version cannot be
written", not "faster than three.js" in raw raster throughput. The
isolation, play-surface and AI-native goals are unaffected, and the project
continues on that basis; the phase-3 lanes and GPU-driven culling are where
the engine pulls ahead on CPU-bound content further.

## 11. Phases

| Phase | Months | Delivers | Evidence |
|---|---|---|---|
| 0 Spike | 0.75 | §10 | the four numbers, published in the platform goal |
| 1 Core | 3 | ECS + reflection, tick + determinism, snapshot, `.ogt`/`.ogb`, cook (glTF, textures, scenes), forward+ PBR with shadows and IBL, input, camera, audio, command channel, probe, hot reload, `og`, the IDL pipeline (d.ts + wiki), project template, two archetype games | gfx references, the probe conformance suite from `origin/packages/contracts`, a Discover prototype rebuilt and played through `og run --commands` |
| 2 Mid-size 3D | 4 | skeletal animation + blend trees + IK, rapier + character controller, navmesh, particles, post stack, terrain, decals, UI toolkit with game styles, MSDF text, Tune | all six Discover archetypes rebuilt; one mid-size 3D showcase; perf baselines in CI |
| 3 Scale | 3 | lanes, GPU-driven culling, streaming, `og run --device`, splats (optional) | the §7 budgets held on the reference scene with lanes on |
| Platform migration | overlaps 1–2 | §5 executor classes; Player top-level for desktop and creator; `@origingame/engine` in the knowledge pack; Wizard deploys the new artifact; Godot distribution and Three.js substrate retired after the last Game migrates or is archived | `bun deploy/ovh/state.ts` drift closed; `verify-production.sh`; image sizes |
| Release (2026-10-22, `docs/release.md`) | after the features | the five packages published at one version; the install test on darwin-arm64, linux-x64 and in an InstaBox machine; the platform's local stack and the M7 acceptance on the published version; the development sandbox on InstaBox (the platform's `instabox` provider and per-class snapshots of the version); E2B images that install the version, production only and last | `release.sh verify --registry --instabox` passes; `pin-<version>.md` on the platform; a development thread in an InstaBox machine, then a production thread on the `engine` class |

**Phase 4 status (2026-10-21).** Phase 4 took the renderer, text, UI shell,
cook, streaming and the lanes past phases 1–3 in one wave of streams
(`docs/phase-1.md`, the Phase 4 sections and the 2026-10-21 main gate).
Of §2.6, every item listed has landed, each with reference frames on
`metal` and `vulkan-cpu` and judged in the browser tiers (with the
WebGL2 tier's reductions §2.6 names): Forward+ with the prepass on
demand (measured, P4 perf), the physical lobes (P4 materials), PCSS,
cascade blending and 8 / 4 shadowed point and spot lights (P4 shadows),
IBL from cooked environment maps (equirectangular, KTX2 cubes, six faces)
with AgX and adapting exposure (P4 ibl-post, cook three and four), the
stylisation primitives as looks (37 presets; P4 look and stylisation
rounds two to four), CPU particle collision against the scene's colliders, kerned,
shaped and bidi text with run-time glyphs (P4 UI text), and every WGSL
module validated and translated per backend in the gate (the wasm ships
WGSL; P4 perf). Of §7: the wasm budget holds per download, the largest
the lanes module at 3,320,578 B (79 %; docs/ci.md, before the wave's later
rounds, which are not measured yet); the reference scene runs at 2.04×
three.js's InstancedMesh on WebGPU and 1.54× on WebGL2 in Chrome on an M4
Pro (`tests/evidence/P4/perf/threejs-spheres.md`); hot reload's worst is
112.8 ms in `og dev` (`tests/evidence/P2/tooling/m4-m5-tooling.md`). Not
measured: the phone first frames and the TS system on a mid-range Android
(the owner's devices, phase-1 owed row 40), the probe `capture` budget and
the executor image's size. Of §12, items 3 to 5 are still the owner's: 3
is delivered in practice (the 2D profile with rapier2d ships its own
publish module, owed row 44) but not marked decided; 4 waits on the
Player's telemetry from the platform; 5 is open (`og run` runs natively
on macOS and Linux, the package ships `darwin-arm64` and `linux-x64`
addons, nothing for Windows). Owed row 44's reading of the wasm budget per
download (§7) is the engine's, written here, and awaits the owner's word.

**Coverage audit, 2026-10-24.** The phase 2 and 3 table in
`docs/phase-1.md` §F3 coverage audit and terrain acceptance maps every
required feature to code and archived checks. The phone, capture and
executor-image budgets above remain unverified. The native-platform
status above is historical: Windows x64 with DX12 shipped in 0.3.0;
registry and platform-runner acceptance are tracked in release F7.
Cloud CI runs only by hand (docs/ci.md §CI); its `perf` lane measures, it does not enforce the perf baselines.

## 12. Open decisions (owner)

1. ~~Licence~~ **Decided 2026-10-17: MIT** (`LICENSE`). The platform services
   are the product; the engine is open.
2. ~~Product mark~~ **Decided 2026-10-17**: composed from the existing master
   symbol (§9); geometry in the platform's `product-marks.ts`.
3. **2D as a first-class profile** (orthographic camera, sprite batching,
   2D physics via rapier2d) in phase 1 or phase 2.
4. ~~WebGL2 fallback lifetime~~ **Decided 2026-11-01: retired.** Quality over
   reach (§8): the web is WebGPU only, with no reduced tier. The original
   question was: keep until WebGPU share on the platform's
   real devices passes a threshold the Player telemetry reports.
5. **Native macOS/Windows developer runs** of `og run` beyond executors (nice
   for engine developers; not a product).

## 13. Alternatives considered

- **Bevy as the base.** Mature wgpu renderer and ECS in Rust, MIT/Apache. Rejected
  because the API changes every release, which defeats the stable API the
  agent's knowledge depends on, its wasm builds are large, and web
  multithreading is not supported. Its crates (wgpu, naga, glam) are used
  directly.
- **Keep Godot, improve the tooling.** The headless footprint, the editor-centric
  data model and GDScript remain; the agent's inner loop stays slow.
- **Three.js with an imposed structure.** Keeps the knowledge advantage but
  leaves JavaScript as the per-frame engine and the iframe as the isolation;
  the performance and play-surface goals are unreachable.
- **A C engine like HZ.** Proven shape, but Rust gives the same wasm output
  with memory safety and a team that already writes it; game code in C would
  put a compiler back into the author's loop.
- **Game code in Rust.** Strongest isolation and performance, rejected for
  compile latency, toolchain size in every sandbox and the author's loop.

## 14. Sources

Observed shapes, not code: the HZ engine's AI-native workflow (command
channel, evidence loop, wiki-as-map, reflection to blobs, one build tool);
wgpu, naga, rapier, meshoptimizer, KTX2/Basis, QOA, MSDF, Clay layout;
the platform's probe protocol 5, play-surface decisions D1–D12, agent-system
decisions D12–D19 and D23, and the platform's design language for the
player shell and the marks only (§9).
