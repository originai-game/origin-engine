# Depth features: rendering depth, the sequencer, the video producer, rooms and probe depth

The feature areas beyond the core engine that close the gaps against
Babylon, Godot 4 and Unity: runtime geometry, the timeline, the video
producer, lockstep rooms, the probe's semantic layer, LOD and reflections,
baked lighting, render targets and custom shaders, 2D skeletal animation
and the math layer. Each area's design is in `docs/design.md` (the section
named below); the IDL (`crates/og-api/api/<module>.ogt`) is the contract.
A type or parameter that has a three.js name takes that name. `debug`,
`expect`, `trace`, `describe`, `save_scene` and the `og render` machinery
are development members, marked so in the IDL and absent from `publish`.

## 1. Runtime geometry, lines, points, debug draw, mesh raycast

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
  update, filtered by `layers` (§8; default mask 1).
- Code: og-render `lines.rs`, `points.rs`, `debug.rs`, `geometry/` (the BVH);
  the `render` and `debug` IDL modules. Design: `docs/design.md` §2.6.

## 2. Paths and the timeline

Cutscenes as an `.ogt` asset the agent reads and diffs; also the edit
decision list the video producer (§3) plays.

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

## 3. The video producer

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

## 4. Rooms and deterministic lockstep

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

## 5. The semantic frame, assertions, trace, write-back, diagnostics

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

## 6. Mesh LOD, reflection probes, screen-space reflections

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

## 7. Baked lighting, irradiance volumes, volumetric fog, area lights, cookies

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

## 8. Render targets, secondary cameras, layers, custom shaders, custom post passes

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

## 9. 2D skeletal animation and the math layer

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
