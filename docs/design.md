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
            │  │  or WebGL2 on OffscreenCanvas), anim, physics, nav, audio mix, │ │
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
  interpolates between the last two ticks. Game code never reads wall-clock
  time: `Time` is a resource with `tick`, `dt`, `elapsed`.
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
  the runtime reads only `cooked/`.
- Reusable production assets come through the platform's asset MCP as today;
  the cook consumes what the agent put in `assets/`.

### 2.6 Renderer **(D)**

wgpu with WGSL shaders cooked per backend (naga). WebGPU is primary; WebGL2
is the fallback through wgpu's GL backend with a reduced feature tier
(no compute: GPU culling and particles fall back to CPU paths, clustered
lighting falls back to a forward tile table built on the CPU).

- Forward+ (clustered), with a depth-normal prepass when an effect reads
  depth or normals (on every frame it cost the 20,000-sphere field two
  thirds of its frame and bought nothing: `tests/evidence/P4/perf/prepass.md`);
  PBR metallic-roughness
  (glTF-compatible), normal, emissive, occlusion, and three.js's physical
  lobes (clearcoat, sheen, transmission, iridescence, anisotropy).
- Cascaded shadow maps for one directional light (opt-in PCSS soft
  shadows and blending across cascade splits); cube faces or two
  dual-paraboloid hemispheres for up to 8 point/spot lights on the full
  tier and 4 on WebGL2, the brightest at the camera first (cube on the full
  tier and paraboloid on WebGL2 by default; `RenderOptions.pointShadows`).
- IBL from an environment map prefiltered at cook time; sky model with
  exposure; height fog.
- Post: bloom, tonemap (ACES or AgX), colour grading LUT, SSAO (GTAO on
  WebGPU), antialiasing by tier: TAA on the full tier (optional
  Catmull-Rom history and sharpening), 4× MSAA on WebGL2, FXAA on any tier
  and where the HDR target cannot be multisampled.
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
- Terrain: clipmap heightfield with splat materials. Decals: deferred-style
  projection in the forward pass.
- Text: MSDF for UI and world text, kerned from the font's GPOS pairs;
  CJK coverage through the platform's subset fonts, and characters outside
  a cooked charset rendered at run time from the font's outlines (`font {
  runtime true }`); bidi (UAX #9) and shaping (Arabic, Indic) through
  rustybuzz, behind og-ui's `shaping` feature.
- Capture: a readback path from the render graph, used by the probe's
  `capture` on both hosts; on the web it reads the OffscreenCanvas target, so
  no `preserveDrawingBuffer`.

Runtime geometry and debug draw (depth S1). Games can make geometry from
attribute arrays (`og.render.bufferGeometry`, three.js BufferGeometry's
attributes, with normals computed when they are missing) and from three.js's
primitives: cylinder, cone, capsule, torus, ring, circle, and a tube along a
centripetal Catmull-Rom curve. A dynamic geometry is rewritten in place
each tick (`updateGeometry`). A made geometry is also a physics mesh
under its handle, so an og.Collider can use it as its `mesh`. Each geometry
has a BVH, built when it is made or updated and lazily for loaded ones;
`og.render.raycast` uses it to test what is drawn rather than the
colliders. Inside a system, the raycast reads the meshes as the probe
recorded them at the tick's start.

Lines and points are one pass (`lines.rs`, `points.rs`,
`shaders/lines.wgsl`). Each segment or point is an instance of a quad in
screen space with a feathered edge, so widths in pixels or in world units
and dashes behave the same on WebGPU and WebGL2. WebGL2 has no wide lines
and WebGPU no line width at all. Material `wireframe` draws its instances'
unique triangle edges through the same pass, and the edges are computed
once per mesh revision. The pass draws in two places:

- **Scene**: in the main pass after the transparent meshes and particles,
  depth tested or not.
- **Overlay**: over the finished frame, after every post effect and before
  the UI. Debug draw (`og.debug`), the debug views and screen lines draw
  here, so they read the same in every capture.

The debug views are a `RenderOptions.debugView` enum (none, physics, nav,
bounds) that later streams add variants to; the command channel's `debug:`
action switches them. The physics view is rapier's debug render of the
last step: collider shapes, joints and contacts. The nav view draws the
grids, navmesh polygons and agents' paths, and the bounds view draws mesh
bounds. Debug draw is rendering: it is not world state, it never feeds the
tick, and the `debug` module is left out of publish builds.

### 2.7 Animation, physics, navigation, audio, UI **(D)**

- **Animation**: glTF skins and clips, a blend tree asset (`.ogt`), state
  machines as data, root motion, additive layers, two-bone and FABRIK IK,
  blendshapes.
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
- **UI**: immediate-mode (Clay-like layout) rendered by the engine; element
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
  for every style (`pack_contrast.rs`). Screen transitions, count-ups and
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
  bones parent first, slots in draw order, `region` and weighted `mesh`
  attachments, one- or two-bone `ik` constraints, and clips of bone
  (rotate, translate, scale), slot (attachment, colour) and deform keys
  with events; keys interpolate linearly or hold (`step`). The cook stores
  the canonical text in an `og.Rig2D` blob, written by hand or imported
  from a DragonBones 5.5 export with its atlas (`dragonbones "x_ske.json"`,
  `atlas "x_tex.json"`). Spine's format is not imported (its licence); the
  module docs of `rig2d.rs` give the mapping a converter follows.
  `og.load.rig2d` registers a rig and its texture (`og.anim.rig2dInfo`
  reads it back; `og.anim.clip` finds a clip by name). `og.Skeleton2D`
  (`rig`, `clip`, `time`, `speed`, `loop`, `flipX`, `layer`, `finished`)
  plays it through the `og.skeleton2d` system, run with the tweens and
  flipbooks after the game's systems; clip events arrive as
  `og.Skeleton2DEvent`; `og.IkTarget2D` moves a constraint's target and
  reads back `reached` and `error`. The pose is a function of the
  component, its Transform and its IK targets, every angle through
  arithmetic-only sincos and atan2, so it is the same bits on both hosts;
  only the playback position is a state block. Extraction draws each
  skeleton as one `og_render::SpriteMesh`: the sprite batch extended to
  triangles (`sprite_mesh.rs`), sorted with the sprites of its layer and
  lit by `og.Lighting2D` as a sprite there.

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
`join`, `leave` and `send`. The events are `joined`, `peer_joined`,
`peer_left`, `message`, `left` and `error`.

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
  around every frame.
- **On the web.** The Worker posts the engine's ops to the page and
  delivers the page's events. The shell's `RoomHost` maps them onto the
  Player's `room.*` (docs/platform-asks.md #16, open). A Player whose
  hello does not list `room.join` leaves the host as `none`, and
  roomCreate and roomJoin are refused.

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
  frame's GPU memory. `og.VideoPlayer` (`video`, `playing`, `time`,
  `speed`, `loop`) is played by the `og.video` system by the tick (not the
  time scale) and keeps the entity's `og.AudioSource` in step: its
  `startOffset` is the video's time, and a seek stops it for a tick and
  starts it again there. A player's state is its component, so a restored
  snapshot plays on as the world that took it; at a 30 Hz tick, tick 90
  shows the source's frame 3 s in (`packages/engine/tests/og-video.test.ts`).
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
- **Start.** Until the room holds `players` peers, each world runs freely
  (the lobby). Then the authority starts lockstep: the authority is the
  lowest peer number, and its settings become the room's. Every other
  peer restores the authority's snapshot, and the seats go to the peers
  in number order.
- **The tick gate.** Tick t runs only when every seated player's input for
  t is in. The gate is og-probe's (`set_gate`, consulted by step, frame
  and advance), so the tick itself is unchanged and a frame that cannot
  run a tick simply runs none.
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
  native processes through a relay that the playtest starts, and both hash
  the same at tick 600.

## 3. API surface

### 3.1 One IDL, two hosts **(D)**

The engine's public API is declared once (`crates/og-api/api.ogt`, the same
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
| `set:<entity>.<component>.<field>=<value>` | a Tune write through the schema |
| `seed:<n>` / `pause` / `resume` / `advance:<ticks>` | determinism controls |
| `describe[:path]` | the frame as data (§3.4): visible entities with screen boxes, depth order and occlusion; UI elements with rects and text; the camera |
| `expect:<operation>` | an assertion at this tick (§3.4); a failure fails the run |
| `save_scene[:path]` | the boot scene's `.ogt` rewritten from the world (§3.4) |
| `quit` | exit |

### 3.4 Probe **(D)**

The engine implements `RuntimeProbe` protocol 5 natively (`hello`, `tick`,
`state`, `advance`, `pause`, `resume`, `input`, `accepted`, `pick`, `entity`,
`bounds`, `patch`, `run`) plus the wire-only members `capture`, `sweep`,
`reload`, `snapshot`, `restore`, `dump`, `commands`. Transport: in-page calls
from the shell (no postMessage hop: the shell calls the Worker through a
request ring), a loopback WebSocket from `og` on native. `publish` builds keep
the player-facing members (`hello`, `tick`, `state`, `pick`, `entity`,
`input`, `capture`) and drop the rest.

Above capture sits the layer that lets an agent check a frame without
reading pixels:

- **The semantic frame.** `describe` (a probe member and the
  `describe[:path]` action) answers the current tick's frame as JSON,
  deterministic for the same world: the visible entities nearest first, each
  with its name, archetype, label, screen-space box and pick point (CSS
  pixels), depth and the fraction of it that others hide; the UI elements
  with their rects and text (from `dump_ui`); the camera. The entities come
  from an entity-id pass: og-render's `id_pass.rs` renders the extracted
  frame (meshes, skinned meshes posed on the CPU, world sprites) at half
  the width and height, a quarter of the pixels, on the CPU, so a host
  without a GPU describes the same frame. `pick` answers from the same pass
  when the game has no picker, so a box `describe` gives contains the point
  `pick` names.
- **Expectations.** `expect` operations run as actions (`expect:<op>`) and
  as a probe program step, and are evaluated at their tick:
  `entity:<id>:within:x,y,w,h` (its described box inside a screen box) or
  `within:x0,y0,z0,x1,y1,z1` (its position inside a world box),
  `event:<Name>[:count]`, `ui:text:[=]<text>`,
  `region:x,y,w,h:color:#rrggbb[:tol]`, `frame:<ref.png>:rmse:<=v` and
  `state:<path><op><value>` over the game's `state()` facts. Each result
  keeps its tick, what was expected and what was observed; a failure is a
  warning in the log. `og run --json` carries them (`expectations`) and
  exits 1 on any failure; a playtest may be expectations alone. A frame
  expectation without a renderer is unchecked, never a failure.
- **Trace.** `og run --trace out.json` writes the run in the Chrome trace
  format (Perfetto opens it): lanes are threads — engine (each tick, its
  systems inside it from the world's clock), render (each frame, its
  render-graph passes), load (each cooked output read), cook (the cook and
  the outputs it cooked). Systems and passes are laid end to end inside
  their parent, since the engine records their lengths. Pass times are CPU
  encoding, not GPU time. The live probe's `state().timings` carries the
  last tick's `systems[]` and the last frame's `passes[]` (name, ms).
- **Scene write-back.** `save_scene` (member, and `save_scene[:path]`)
  rewrites the boot scene's `.ogt` from the world, line by line: a field
  line changes only when its value, rounded as the cook rounds it, differs;
  a field `set` or `patch` tuned that the text lacks gets a line (or a
  component block); comments, order and spelling stay. Cooking the result
  loads the same world, and saving again changes nothing. Resources are
  not scene text: a tuned resource is reported as skipped.
- **Diagnostics as data.** `og cook --report` writes
  `og-out/cook-report.json`. For each asset it records triangles,
  vertices and LODs, textures with sizes and formats, bones, clips, bytes
  cooked and budget flags. `og doctor --project <dir> [--json]` checks it
  against the budgets project.ogt sets (`budgets { triangles vertices
  textureSize bones clips assetBytes totalBytes }`, each with a default)
  and prints findings with severity, asset, budget, limit, value and fix.

### 3.5 Hot reload **(D)**

`reload` snapshots the world, swaps the game bundle (bun rebuilds in tens of
milliseconds), re-registers components by name and restores. A component
whose layout changed is migrated field by field through the schema; a
removed one is dropped with a warning in the reload report.

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

Measured in the spike (§10) and gated in CI afterwards.

| Budget | Target |
|---|---|
| engine wasm, brotli | ≤ 4 MB core; ≤ 6 MB with physics and animation |
| first frame, mid-range Android, 4G, small game | ≤ 2.0 s click to first frame (the play-surface budget) |
| first frame, iPhone 13, Wi-Fi, mid-size 3D scene | ≤ 1.5 s |
| TS system over 10,000 entities (one archetype, 2 components) | ≤ 1.0 ms per tick on a 2022 mid-range Android |
| reference scene (500 instanced objects, 2 shadowed lights, 20 skinned characters, PBR, post) | ≥ 1.5× the frame rate of the same scene in Three.js on the same device |
| probe `capture` 1280×720 | ≤ 30 ms native, ≤ 60 ms web |
| hot reload | ≤ 300 ms from bundle write to first new frame |
| headless executor image (`engine`) | ≤ 600 MB |

The hot-reload budget is rebuild and swap. `og dev`'s type check of the same
save runs beside the rebuild in a process of its own and is reported apart
(`check.ms`, 33–48 ms on the template, `tests/evidence/tsrs/wave-2.md`); it
never delays or refuses a reload, so it is outside the 300 ms.

The wasm budget is per download: a page downloads one engine module, the
publish variant of its game's physics profile (3D or 2D) or, on a
cross-origin isolated page, the lanes variant, so each module is held to
it and the figure that counts is the largest one a player downloads, never
the sum of the modules a build ships (docs/web-export.md §Variants,
docs/ci.md §Updating a budget).

## 8. Platform matrix

| | WebGPU (Chrome, Edge, Safari 26+, recent Firefox) | WebGL2 fallback | native Linux (executor) | native macOS (developer) |
|---|---|---|---|---|
| renderer | full | no compute: CPU culling/particles, tiled lights, no GTAO | full with a Vulkan device (a GPU; lavapipe set aside), else none: the simulation, the probe and the playtests without their frames | full (Metal) |
| lanes | with COOP/COEP | with COOP/COEP | rayon | rayon |
| audio | AudioWorklet | AudioWorklet | cpal or null | cpal |
| capture | readback | readback | readback | readback |
| determinism | yes | yes | yes | yes |

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
  `fran0220/origin-engine` (`docs/release.md` §2).
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

## 12. Open decisions (owner)

1. ~~Licence~~ **Decided 2026-10-17: MIT** (`LICENSE`). The platform services
   are the product; the engine is open.
2. ~~Product mark~~ **Decided 2026-10-17**: composed from the existing master
   symbol (§9); geometry in the platform's `product-marks.ts`.
3. **2D as a first-class profile** (orthographic camera, sprite batching,
   2D physics via rapier2d) in phase 1 or phase 2.
4. **WebGL2 fallback lifetime**: keep until WebGPU share on the platform's
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
