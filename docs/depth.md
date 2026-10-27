# Depth wave: the gaps against three.js, Babylon, Godot and Unity, and the video producer

Decided by the owner on 2026-10-22, after the comparison recorded in the
thread of that day: the engine's coverage already exceeds three.js (an ECS,
physics, navigation, audio, UI, AI, determinism and the probe, none of
which three.js has) and what remains against Babylon, Godot 4 and Unity is
rendering depth, a sequencer, multiplayer, the next layer of AI-native
tooling, and a video producer built on the engine's determinism. This
document is the execution package: nine streams, each one agent thread on
its own branch, merged by the coordinator (this repository's rules,
`AGENTS.md` §Build and gate discipline). It is row F8 of `docs/release.md`.

The design stays `docs/design.md`; each stream adds the section its feature
needs there (named below) in its own commit, the IDL (`crates/og-api/api.ogt`)
is the contract, and the generated outputs (`og api gen --wiki`: the `.d.ts`,
`packages/engine/src/generated/abi.ts`, the wiki) are committed with every
IDL change. The three.js vocabulary rule holds: a type or parameter that has
a three.js name takes that name.

## 0. How the wave is run

- **One stream, one branch, one thread.** Branch `depth/<stream>` from main.
  The thread commits on its branch as it goes (small commits, never a pile
  in the working tree) and pushes the branch to `origin`; it never merges
  into main, never opens a pull request. The coordinator (the session on
  `macrunner`, `/Volumes/app/origin-engine`) merges each branch into main,
  regenerates the API outputs, records the Metal reference frames, runs the
  functional lane (`scripts/check.sh functional`), writes the CHANGELOG
  entry and the round's section in `docs/phase-1.md`, and pushes main.
- **An agent's loop** is `cargo check -p <crate>` and the unit tests of the
  crate it edits, `og api check` after an IDL change, and `og run --headless
  --commands` of the station or template it adds (the orbs render on
  lavapipe: slow, CPU only, a smoke check; read at 640×360 where it
  can be, never recorded as a reference). An agent never runs
  `scripts/check.sh`, a test lane, `og test`, web-reference, a wasm or
  release build. A gfx test an agent adds names its reference frame and
  leaves the `metal` file to the coordinator, who records it at the merge
  and reads it; the agent's evidence is its lavapipe frame, read and
  described in its notes.
- **Shared files.** `api.ogt` is one file: a stream adds its own module or
  appends inside the module it extends, never reorders. og-render's frame
  graph, the lit WGSL and `og-cli/src/main.rs` are touched by several
  streams: new code goes in new files, a touch of a shared file is the
  smallest hook that reaches it, and the stream's notes list every shared
  file it changed so the coordinator merges in the right order (§10).
- **Evidence** is `tests/evidence/depth/<stream>/NOTES.md`: what landed
  with commits, the frames read and described, the numbers, what was left
  out and why. Skills (`packages/knowledge/skills/*/SKILL.md`) teach the
  feature in the stream's commit; the wiki is generated.
- **Publish builds** keep what the player needs; `debug`, `expect`,
  `trace`, `describe`, `save_scene` and the `og render` machinery are
  development members, marked so in the IDL.
- **Determinism is not negotiable** in any stream: no wall-clock reads in
  the tick, randomness from `og.rng`, rendering never feeds back.

## 1. S1 `geometry`: runtime geometry, lines, points, debug draw, mesh raycast

The gap that blocks procedural levels, 2D polygons, tool-made meshes and
debug visualisation. three.js vocabulary throughout.

- `og.render.bufferGeometry(world, desc: BufferGeometryDesc): Result<Geometry>`:
  `positions` (Float32Array, xyz), optional `normals`, `uvs`, `colors`,
  `tangents`, `indices` (Uint32Array), `dynamic bool`; missing normals are
  computed. `og.render.updateGeometry(world, geometry, desc)` rewrites a
  dynamic one in place (same vertex count or a reallocation, documented).
  `og.render.geometryInfo(world, geometry): Result<GeometryInfo>` (counts,
  bounds). A buffer geometry is a `Collider.mesh` through `og.physics.setMesh`
  as a loaded one is.
- Primitives beside box, plane and sphere, three.js parameters and
  defaults: `cylinderGeometry`, `coneGeometry`, `capsuleGeometry`,
  `torusGeometry`, `ringGeometry`, `circleGeometry`, `tubeGeometry` (along
  points, Catmull-Rom through them; S2's `og.path.sample` feeds it with
  points so the two streams do not depend on each other).
- Lines and points: immediate mode as `og.render.sprite`:
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
  (a `RenderOptions.debugView` enum the other streams add variants to).
  Stripped in `publish`.
- `og.render.raycast(world, ray, RaycastGeometryDesc): Result<GeometryHit[]>`
  against meshes (not colliders): a BVH per geometry built at load and on
  update, `layers` when S8 lands (the field exists from the start, mask 1).
- Crates: og-render (new `lines.rs`, `points.rs`, `debug.rs`, `geometry/`
  with the BVH), og-runtime, og-api (`render` module additions, new `debug`
  module), hosts, `templates/fx-showcase` station `geometry` with its
  playtest, og-render gfx tests for lines, points, wireframe, a procedural
  mesh and the debug overlay.
- Design: `docs/design.md` §2.6 gains a paragraph on runtime geometry and
  debug draw.
- Acceptance: the station draws a procedural terrain-like mesh made each
  tick (dynamic), a tube along a spline, dashed screen lines, a point cloud
  and the physics overlay; `og run --commands "2:debug:physics;3:screenshot"`
  shows colliders over the meshes; raycast against the procedural mesh
  answers the hit the collider raycast answers (a unit test with both).

## 2. S2 `timeline`: paths and the sequencer as data

Unity's Timeline and Godot's AnimationPlayer for cutscenes, as an `.ogt`
asset the agent reads and diffs; it is also the edit decision list the
video producer (S3) plays.

- Paths: `og.path.create(world, PathDesc): Result<Path>` (`points`,
  `closed`, `kind` `catmullRom | bezier | linear`, `tension`),
  `og.path.sample(world, path, t): PathSample` (position, tangent, up, at
  arc-length parameter 0..1), `og.path.length`, `og.path.nearest`;
  `og.PathFollower` component (`path`, `speed` units per second, `loop`,
  `align` to the tangent, `offset`), stepped by the tick. `og.CameraRail`
  keeps its API and reads a path handle too (`rail.path`).
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
    game's registered command), so a timeline drives the game without new
    API;
  - `transition`: `cut | fade | wipe | dissolve` with duration, over the
    screen transition of og.ui;
  - `marker`: named times (`og.TimelineMarker` event when passed).
- `og.Timeline` component (`asset`, `playing`, `time`, `speed`, `loop`),
  `og.timeline.play / pause / seek / stop`, events `og.TimelineStarted`,
  `og.TimelineMarker`, `og.TimelineEnded`. Seeking is exact: the keys are
  evaluated at the tick's time, never accumulated, so S3 can start at any
  frame. Command-channel action `timeline:play:<name>`, `timeline:seek:<s>`.
- Crates: new `og-timeline` (evaluation), og-cook (timeline, `.srt`,
  paths in scenes), og-api (new `path` and `timeline` modules), og-runtime,
  hosts; `templates/game` gains `timelines/intro.ogt` and its playtest.
- Design: `docs/design.md` §2.7 gains "Timeline and paths".
- Acceptance: the template's intro plays a camera move along a path, a
  character clip, a caption from an `.srt`, a sound and a fade, and
  `og run --commands "0:timeline:play:intro;4:screenshot;8:screenshot"`
  shows the caption and the camera where the keys put them; seeking to 4 s
  then capturing equals playing to 4 s (a unit test on world hashes).

## 3. S3 `video`: the producer

The engine already has what the "video is code" pipelines bolt on
(deterministic `seek(t)`, headless runs, offline audio); this stream makes
the output a video and lets footage in.

- `og render [project] --commands "..." | --timeline <name> --fps 30
  --from <s> --to <s> --size WxH [--out dir | out.mp4 | out.webm] [--frames
  png|jpg] [--audio] [--contact-sheet <cols>] [--jobs N] [--json]`: a native
  headless run at the project's tick (the fps must divide the tick rate,
  else an error names the nearest), one readback per frame through the
  capture hooks, frames piped to `ffmpeg` (found by `og doctor`; absent,
  frames only) as H.264 or VP9, the capture mixer's WAV muxed with `--audio`.
  `--jobs N` splits the range: each job restores the snapshot the leader
  took at its first tick and renders its range; `ffmpeg` concatenates.
  `--contact-sheet` writes a grid PNG with timestamps (the agent's review
  surface) and `frames.json` (tick, time, sha256 per frame, the audio's
  sha). Two runs of one script produce byte-identical frames (a test).
  `--timeline` is `--commands "0:timeline:play:<name>"` plus the timeline's
  duration as the range; it works once S2 lands and is documented as such.
- Video in: `type video` asset (`.ogt` naming an `.mp4`, `.webm` or `.mov`,
  `fps`, `maxSeconds`, `size`), cooked by `ffmpeg` into a texture array of
  frames (KTX2, the cook reports the bytes; a budget warning past 64 MiB)
  and its audio as a sound; `og.load.video: Result<Video>`,
  `og.load.videoInfo`, `og.VideoPlayer` component (`video`, `playing`,
  `time`, `speed`, `loop`) whose current frame is the `Texture` of a
  material map, an `og.Sprite` or a UI image through `og.VideoPlayer.texture`,
  with its sound on an `og.AudioSource` kept in step by the tick.
- Crates: og-cli (new `render.rs`, `contact_sheet.rs`), og-runtime and
  og-probe (every-frame capture without the probe's round trip), og-cook
  (`video`), og-render (a texture-array source for the player; new file),
  og-api (`video` module; `VideoPlayer`), `templates/tools` gains a
  `render` example and a `reviews` note.
- Design: `docs/design.md` gains §2.9 "The video producer" (frames, audio,
  footage, the review surface, the CPU-only sandbox's cost).
- Acceptance: `og render templates/game --commands "0:timeline:play:intro"
  --fps 30 --from 0 --to 8 --out intro.mp4 --audio --contact-sheet 6` makes
  an 8 s file with sound and a sheet; two runs match frame for frame; a
  cooked video plays on a sprite in a gfx test with its frame at tick 90
  being the source's frame 3 s in.

## 4. S4 `rooms`: the room ABI and deterministic lockstep

Design §2.8 promises `og_room_*`; the API has none. The engine's fixed tick,
owned RNG and whole-world snapshot make lockstep the natural netcode, and
replay its recording.

- Host ABI and `og.platform.roomCreate(desc)`, `roomJoin(id)`, `roomLeave`,
  `roomSend(bytes | json, to?)`, `roomPeers`, events `og.RoomJoined`,
  `og.RoomPeerJoined`, `og.RoomPeerLeft`, `og.RoomMessage`; answered on the
  web by the shell (the platform's rooms, `docs/platform-asks.md` gains the
  ask with the message shapes) and on native by `og relay [--port]`, a
  local WebSocket relay, so two `og run --room ws://127.0.0.1:P/<room>`
  processes play together on one machine.
- Lockstep in the engine: `og.Lockstep` resource (`enabled`, `players`,
  `localPlayer`, `inputDelay` ticks, `hashEvery` ticks) and `og.PlayerInput`
  (a reflected component the game declares per player: the engine
  serialises it, exchanges it through the room, and advances tick t only
  when every player's input for t is in; a game reads inputs from the
  component, never from `og.Input` directly, when lockstep is on). Late
  join through a snapshot from a peer; the world hash every `hashEvery`
  ticks; on a mismatch `og.Desync` (tick, peers) and the probe's `desync`
  dump: the first divergent tick and the component-level diff of the two
  snapshots, the AI-readable answer to "what is not deterministic".
- Crates: new `og-net` (lockstep, relay client, the hash), og-cli (`relay`),
  og-host-web and og-host-node shims, og-api (`platform` additions, new
  `net` module), `templates/archetypes` gains a two-player `arena` with its
  playtest driving two native processes through the relay.
- Design: `docs/design.md` §2.8 gains the rooms contract and a §2.10
  "Lockstep".
- Acceptance: two native runs through the relay with scripted inputs end
  with identical hashes at tick 600 (a test); a copy of the template that
  reads `Math.random()` in a system triggers `og.Desync` and the dump names
  the component and the first tick.

## 5. S5 `probe-depth`: the semantic frame, assertions, trace, write-back, diagnostics

The layer above capture that makes the agent's loop reliable.

- `describe` (probe method and `describe:<out>` command): the visible
  entities with screen-space boxes, depth order and occlusion fraction
  (from an entity-id pass rendered at a quarter of the frame; new file in
  og-render), their names, archetypes and labels; the UI elements with
  rects and text (from `dump_ui`); the camera. JSON, deterministic.
- `expect` operations in probe programs and as command-channel actions:
  `entity <name> within <box>`, `event <name> [count]`, `ui text "<s>"`,
  `region x,y,w,h color #rrggbb tolerance`, `frame <ref.png> rmse <= v`,
  `state <path> == <value>` (the game's `state()` facts); a failed
  expectation is reported with the tick and the observed value, `og run
  --json` carries them and exits non-zero on any failure.
- `og run --trace out.json`: Chrome trace format, per tick a span per
  system, per render pass, per load and cook event, lanes as threads;
  `ProbeTimings` gains `systems[]` and `passes[]` (name, ms) for the live
  probe.
- Scene write-back: probe `save_scene` and command `save_scene[:path]`:
  the entities that came from the scene and the tuned resources written
  back to the scene's `.ogt` text, line-based, keeping comments and order,
  only the lines whose values changed; a round-trip test (load, change,
  save, load) is exact.
- Diagnostics as data: `og cook --report` writes `cook-report.json` (per
  asset: triangles, vertices, LODs, texture sizes and formats, bones,
  clips, bytes cooked, budget flags) and `og doctor --project <dir>` reads
  it with the project's budgets (`project.ogt`) and prints the structured
  findings `--json`.
- Crates: og-probe, og-runtime (schedule timing), og-cli, og-render (the
  id pass), og-cook (the report), og-api (`probe` module additions).
- Design: `docs/design.md` §3.4 gains the semantic frame, expectations and
  trace.
- Acceptance: a playtest of `templates/game` written as `expect` actions
  replaces its TS assertions and fails on a deliberately broken copy;
  `describe` on the template's first frame lists the player, the floor and
  the HUD with boxes that contain the pixels `pick` answers at their
  centres; a trace of 300 ticks opens in Perfetto (checked by schema).

## 6. S6 `render-lod`: mesh LOD, reflection probes, screen-space reflections

- LOD: the cook simplifies every model mesh with meshoptimizer into levels
  (`type model` `lods [1, 0.5, 0.25]` by default, `MSFT_lod` levels kept
  when the glTF has them), `og.Model.lodBias`, `RenderSettings.lodBias`;
  the renderer picks by projected size with hysteresis; `dump_entity`
  shows the level; `debugView lod` colours by level (S1's enum: add the
  variant in this stream's commit).
- `og.ReflectionProbe` component (box extents, `resolution`, `mode`
  `baked | realtime`, `interval` ticks); box-projected sampling in the lit
  shader blended with the environment map; `og bake probes [project]`
  renders the baked ones on the native host and writes them under the
  project's cook cache, loaded when present, baked lazily on first use
  otherwise (and always on the realtime path).
- SSR on the full tier: `PostEffects.ssr`, `ssrMaxDistance`, `ssrThickness`,
  `ssrSteps`, `ssrFade`; WebGL2 reports it absent in `RenderCapabilities`
  and falls back to probes.
- Crates: og-cook (lod), og-render (`lod.rs`, `probes.rs`, `ssr.rs`), og-cli
  (`bake_probes.rs`, one dispatch line in `main.rs`), og-api, fx-showcase
  station `reflections`, gfx tests for lod levels, a probe-lit interior
  and ssr on a wet floor.
- Design: §2.6 gains LOD, probes and SSR.
- Acceptance: the showcase's 2,000-tree field renders with the frame's
  triangle count in `ProbeTimings` falling by at least 3× at distance
  against lod off; the interior's chrome sphere shows the room, not the sky.

## 7. S7 `render-light`: baked lighting, irradiance volumes, volumetric fog, area lights, cookies

- `og bake lightmaps [project]`: a CPU path tracer (progressive, native,
  no GPU needed) over the scene's static models and lights, lightmap UVs
  from xatlas, the result as KTX2 per scene with `og.Model` reading it
  automatically for static entities (`static true` in the scene line);
  `og.LightVolume` component (box, spacing) baked to spherical harmonics
  that dynamic objects sample; a scene's baked data is loaded when present,
  the scene renders unbaked otherwise and `doctor --project` says so.
- Volumetric fog: `og.Fog` gains `volumetric`, `density`, `scattering`,
  `anisotropy`, `height`; a froxel grid lit by the sun with shadows and by
  the clustered lights on the full tier (light shafts come out of it);
  a cheaper raymarch on WebGL2, reported in `RenderCapabilities`.
- Area lights: `og.RectLight` component (width, height, color, intensity,
  two-sided) with LTC shading; `SpotLight.cookie` and `PointLight.cookie`
  (a `Texture`, a cube for point) projected.
- Crates: og-render (`lightmap.rs`, `volumetric.rs`, `area.rs`; the lit
  WGSL's hooks for lightmap and SH sampling), og-cook or a new `og-bake`
  crate for the tracer and xatlas, og-cli (`bake_lightmaps.rs`), og-api,
  fx-showcase station `baked`, gfx tests.
- Design: §2.6 gains baked lighting, volumetrics and area lights.
- Acceptance: the showcase's Cornell-box station baked in under two minutes
  on an M-series Mac, its indirect red and green bleeding visible in the
  frame and absent with baking off; the volumetric station's shafts through
  a window move with the day-night time.

## 8. S8 `render-targets`: render to texture, secondary cameras, layers, full custom shaders, custom post passes

- `og.render.renderTarget(world, RenderTargetDesc): Result<RenderTarget>`
  (width, height, `format`, `depth`), its colour as a `Texture` for any
  map, sprite or UI image; `og.CameraView` component: a secondary camera
  (the `og.Camera` fields) rendering into a target or into a `viewport`
  rect of the frame (split-screen), with `layers` mask, `order`, `clear`,
  `every` ticks; `og.Mesh.layers`, `og.Sprite.layers`, `og.Camera.layers`
  (three.js layers, bit masks, default 1); `og.Mirror` component: a planar
  reflection as a mirrored CameraView into a target the material reads.
- Full custom shaders: `type shader` gains `mode full` with `vertex` and
  `fragment` entries over a documented binding contract (camera, model,
  skin, params, up to eight textures), `lit` or `unlit`, shadows and the
  prepass drawn through its vertex stage; cooked per backend as the hooks
  are.
- Custom post passes: `type post` asset (a WGSL fragment over the HDR
  colour, depth, normals, the previous pass, params) and
  `PostEffects.custom` (an ordered list of `Post` handles, params set by
  `og.render.setPostParam`); a pass that fails to compile is reported and
  skipped.
- Crates: og-render (`targets.rs`, `views.rs`, `custom.rs`, `post_custom.rs`),
  og-cook (shader modes, `type post`), og-api, fx-showcase stations
  `views` (a security monitor, a mirror, split-screen) and `shaders`, gfx
  tests.
- Design: §2.6 gains targets, views, layers and custom shaders.
- Acceptance: the monitor station shows the room from the second camera on
  the screen mesh and the mirror reflects the character; a full custom
  shader draws a dissolve with shadows following the cut; a custom post
  pass applies a CRT curve over the frame; WebGL2 is reported where a tier
  limit applies, never by a missing member.

## 9. S9 `two-d-math`: 2D skeletal animation and the math layer

- 2D skeletal: `type rig2d` asset in the engine's text (bones, slots,
  attachments: region, mesh with weights, IK constraints, clips with bone,
  slot and deform keys) cooked from that text or imported from DragonBones
  JSON (an open format); `og.Skeleton2D` component (`rig`, `clip`, `time`,
  `speed`, `loop`, `flipX`), `og.Skeleton2DEvent` for clip events, drawn
  through the sprite batch extended to triangles for mesh attachments, lit
  by `og.Lighting2D`; `og.anim.rig2dInfo`. Spine's format is not imported
  (its licence); the text format documents the mapping for a converter.
- The math layer: three.js's methods as pure functions over the plain
  structs, declared in the IDL's `math` module and implemented in
  TypeScript (the stream finds how an IDL function is implemented on the
  TS side, as `og.anim.clip` or the systems are): `vec3.add, sub, scale,
  dot, cross, length, normalize, lerp, distance, applyQuaternion`, `vec2`
  likewise, `quat.multiply, slerp, fromEuler, invert, rotate`, `mat4.multiply,
  invert, transformPoint, transformDirection`, `Box3`, `Sphere`, `Plane`,
  `Frustum` (`containsPoint`, `intersectsBox`, from the camera), `Ray`
  (`intersectBox`, `intersectSphere`, `intersectPlane`, `intersectTriangle`),
  `color.fromHex, toHex, lerp, fromHsl`. No classes: data stays plain so
  columns and reflection are untouched.
- Crates: og-anim (2D), og-cook (rig2d, DragonBones), og-render (the sprite
  batch's triangles; a new file plus the smallest hook), og-api (`anim`
  and `math` additions), `packages/engine/src/math.ts`, `templates/game-2d`
  gains a rigged character and its playtest.
- Design: §2.7 gains 2D skeletal animation; §3.2 the math layer.
- Acceptance: the 2D template's character walks with a mesh-deformed cape
  and an IK-planted foot on a slope; a frame at tick 60 is the reference;
  the math functions have unit tests against three.js's documented results
  (a table of cases, not a port of its tests).

## 10. Merge order and the coordinator's checks

Merged in this order, each through the functional lane before the next:
S1 (the debug enum and raycast others reference), S2, S3, S5, S9, S4, S6,
S8, S7 (the three render streams last, S7 touching the lit shader most).
At every merge the coordinator regenerates the API outputs, resolves the
generated files by regeneration, records the Metal references the stream
named, reads every new frame, writes the CHANGELOG lines for the IDL
changes, and appends the round to `docs/phase-1.md`. The wave's end runs
`scripts/wave-end.sh` once, with the size and frame-time numbers in that
section; a number past its budget is recorded as an owed row, never a
reason to hold a functional merge.
