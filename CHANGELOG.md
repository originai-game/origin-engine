# Changelog

One version across `@origingame/engine` (the knowledge inside it as
`knowledge/`), `@origingame/engine-darwin-arm64`, `@origingame/engine-linux-x64`
(`@origingame/engine-win32-x64` from F7) and `@origingame/create-game`
(docs/release.md §3). Before 1.0.0 a minor
bump carries API additions or changes, one line each, written at the merge
that changes `crates/og-api/api.ogt`; a patch bump carries fixes and frames.

## Unreleased

- Windows x64 (F7, merged after 0.2.0 was published, so the package first
  ships with the next version): build and pack the MSVC `og.exe` and napi
  addon as `@origingame/engine-win32-x64`, with DX12 rendering, embedded
  DXC and the same TypeScript API. `og check` deliberately uses the pinned
  tsgo/Bun fallback, reported by doctor. Windows paths, source line
  endings, HDR fixture checkout bytes and knowledge mount permissions are
  portable. The tarball install test passes on Windows 11; the five gfx
  tests that compare DX12 frames against the Metal references differ only
  by edge and noise dither (reviewed by the coordinator, nothing moved; no
  DX12 reference tier). Evidence: `tests/evidence/release/windows-x64.md`.
  `@origingame/engine@0.2.0` on the registry does not reference the
  win32-x64 package; the engine's `optionalDependencies` pin it from this
  commit on.

UI styles (branch `ui/style`): a game's UI is the game's own, never the
platform's. API changes (a minor bump; breaking before 1.0.0):

- og-ui owns its values (`crates/og-ui/src/theme.rs`); the vendored
  platform `tokens.json` and its generator are gone. Without a style the UI
  draws in `prototype`, a placeholder (greys, radii 1/2/1/3, a cyan focus).
- Palette roles renamed to the engine's 33 (`ThemeColor`: `backdrop`,
  `panel`, `ink`, `inkMuted`, `inkFaint`, `primary`, `onPrimary`,
  `primarySoft`, `control`, `line`, `hud`, ...); `accent` is `primary`
  (`Tone`, `MeterKind`, `Ink`); `UiDesc.primaryColor` replaces the accent
  and hue (`HueName` removed); `DensityMode` is `auto`, `mouse`, `pad`,
  `touch`; radii are `small`, `control`, `bar`, `panel`.
- The skin is a **style**: `og.load.style` (was `og.load.skin`),
  `UiDesc.style`, `UiTheme.style`, `LookInfo.style`, the `Style` handle,
  `StylePart` with `hud` (was `capsule`), blob `og.UiStyle`, `type style`
  files that name every palette role and radius (`type skin` and `base`
  are refused with the fix), `og style pack` (was `og skin pack`).
- A look's `ui` line names the style: a built-in by slug
  (`templates/styles/{pixel,cartoon,fantasy,scifi,paper,neon}`, cooked
  under `@styles/`) or the game's own; every catalogue preset names one
  (`looks/catalogue.json`'s `ui`), so a game extending a preset is dressed.
- `og.ui.actionButton` is square, cornered by `radius.control` (was a
  circle); the cartoon style's corners drop from 10/18 to 6/10 and its
  primary deepens to `#8c360f` (7:1 as drawn).
- `dump_ui` carries `style` ("" for prototype); `og polish` lists a UI
  style finding for a game left in prototype.
- The kit's cutscene draws its letterbox bars black in every style (at
  the merge they took the `backdrop` role, which a paper style turned
  into cream panels over Kestrel Stair's intro).

Depth wave 1 (`docs/depth.md` §1–§3, merged 2026-10-23), API additions (a
minor bump, §3), by stream:

- `geometry` (S1), module render: `og.render.bufferGeometry`,
  `updateGeometry` (dynamic geometries only) and `geometryInfo`, with
  `BufferGeometryDesc` and `GeometryInfo`; a made geometry is also a
  `Collider.mesh`. The three.js primitives `cylinderGeometry`,
  `coneGeometry`, `capsuleGeometry`, `torusGeometry`, `ringGeometry`,
  `circleGeometry` and `tubeGeometry` (centripetal Catmull-Rom) with their
  desc structs. Immediate `og.render.lines` (`LinesDesc`, `LineMode`) and
  `og.render.points` (`PointsDesc`); the retained components `og.Lines`
  and `og.Points`. `og.render.raycast` (`RaycastGeometryDesc`,
  `GeometryHit`) through a BVH per geometry, over `og.Mesh` and `og.Model`.
  `StandardMaterialDesc.wireframe`. `RenderOptions.debugView` is the new
  `DebugView` enum (`none`, `physics`, `nav`, `bounds`), also the command
  channel's `debug:<view>`. New module `debug` (`publish false`):
  `og.debug.line`, `box`, `sphere`, `arrow`, `axes`, `text`, `clear`,
  `setView`/`view`, `setEnabled`/`enabled`, with `DebugDrawDesc`.
- `timeline` (S2), new modules `path` and `timeline`: `og.path.create`
  (`CurveDesc`: catmullRom, bezier or linear, closed, tension), `find` (a
  scene's `path "<name>" { ... }`), `sample` (`PathSample`), `length` and
  `nearest` (`PathNearest`), the `Path` handle, `PathKind` and the
  `og.PathFollower` component; `og.CameraRail.path` follows a path instead
  of its points. `og.load.timeline` loads an `.ogt` of `type timeline`
  (the `TimelineAsset` handle); `og.Timeline` plays it with
  `og.timeline.play`, `pause`, `seek`, `stop` and `info` (`TimelineInfo`,
  `TimelineTrackInfo`), raising `TimelineStarted`, `TimelineMarker` and
  `TimelineEnded`; the command channel's `timeline:play:<name>`,
  `timeline:seek:<s>[:<name>]`, `timeline:pause`, `resume`, `stop`. A
  timeline's time is a whole count of thousandths of a tick, so a seek
  hashes as a play. The desc is `CurveDesc`, not `PathDesc`, which og.nav
  already declares.
- `video` (S3), new module `video`: `og.load.video` and `videoInfo` (the
  `Video` handle, `VideoInfo`) for an `.ogt` of `type video`, cooked by
  ffmpeg into a KTX2 texture array and a QOA sound; the `og.VideoPlayer`
  component whose frame is a Texture and whose tick keeps the sound in
  step. `og render` (native only): `--commands`/`--timeline`, `--fps`
  (must divide the tick rate), `--from/--to`, `--size`, `--out
  dir|.mp4|.webm`, `--frames png|jpg`, `--audio`, `--contact-sheet`,
  `--jobs`, `--json`, with a `frames.json` of sha256 per frame; two runs
  are byte-identical and `--jobs` matches one job. `og doctor` reports
  ffmpeg and ffprobe.

Depth wave 2 (`docs/depth.md` §4, §5 and §9, merged 2026-10-23), API
additions (a minor bump, §3), by stream:

- `probe-depth` (S5), module probe: `describe` (the frame as data, a
  `Description` of `DescribedEntity` boxes with depth and occlusion from a
  CPU entity-id pass at a quarter of the pixels, `DescribedUi` and
  `DescribedCamera`); `expect` with six operations (`ExpectOperation`),
  as `expect:` script steps, a program op or members; `expectations`
  (`ExpectReport`, `ExpectResult`, `ExpectStatus`); `saveScene`
  (`SavedScene`: the boot scene's `.ogt` written back line by line from
  the world, comments, order and spelling kept); `state().timings.systems`
  and `.passes` (`ProbeTiming`). `og run --json` carries the expectations
  and exits 1 on a failure, and a playtest may be `expect` steps only.
  `og run --trace out.json` writes the run in the Chrome trace format;
  `og cook --report` writes `og-out/cook-report.json` and `og doctor
  --project <dir> [--json]` checks it against project.ogt's `budgets { }`.
- `two-d-math` (S9), new module `math` (`host false`): three.js's methods
  as pure functions over the plain structs, with `Box3`, `Sphere`, `Plane`
  and `Frustum`. Module anim: `og.load.rig2d` (the `Rig2D` column, an
  `.ogt` of `type rig2d` or a DragonBones 5.5 export), the `og.Skeleton2D`
  and `og.IkTarget2D` components, the `og.Skeleton2DEvent` event,
  `og.anim.rig2dInfo` (`Rig2DInfo`, `Rig2DClipInfo`) and `clip`; a
  skeleton draws as sprite meshes in the sprite order. New template
  `templates/game-2d` (the lamp keeper).
- `rooms` (S4), module platform: `roomCreate`, `roomJoin`, `roomLeave`,
  `roomSend`, `roomPeers`, `roomInfo` and `roomMessage` (`RoomDesc`,
  `RoomPeer`, `RoomInfo`, `RoomMessageData`; the events `RoomJoined`,
  `RoomPeerJoined`, `RoomPeerLeft`, `RoomMessage`). New module `net`:
  `og.Lockstep`, the `og.PlayerInput` component, the `og.Desync` event,
  `submit`, `status` and `hash` (`InputDesc`, `LockstepSeat`,
  `LockstepStatus`); the probe's `dump_desync` (`DesyncDump`) names the
  first divergent tick, entity, component and field with both values.
  `og relay [--port] [--host]` and `og run --room
  ws://127.0.0.1:P/<room>`; `PlaytestDesc.room` runs a playtest as several
  processes through a relay it starts. New archetype
  `templates/archetypes/arena` (Coin Duel). On the web, rooms go through
  the Player's `room.*` (platform-asks #16): until the platform lands it,
  `roomInfo` answers `host: none`.

Depth wave 3 (`docs/depth.md` §6–§8, merged 2026-10-23), API additions (a
minor bump, §3), by stream:

- `render-lod` (S6), module render: the cook makes LOD levels of every
  model (`lods 1 0.5 0.25` by default, `lodError`; glTF `MSFT_lod` levels
  kept; `cook --report` lists them) and the renderer picks one by projected
  size with hysteresis. `RenderSettings.lodBias` and `og.Model.lodBias`
  (0 draws the full mesh), `DebugView.lod` (the `debug:lod` command tints
  by level), `dump_entity`'s `render.lod`, `ProbeTimings.triangles`.
  `og.ReflectionProbe` (`size`, `resolution`, `mode` `ReflectionProbeMode`
  baked or realtime, `interval`, `intensity`, `blendDistance`, `visible`;
  `MAX_REFLECTION_PROBES` 8): six faces captured in a `reflection-probes`
  pass, GGX-prefiltered, box-projected and blended into the lit shaders.
  `og bake probes [project] [--commands] [--at] [--size]` writes
  `cooked/probes/<key>.ogb`, which the runtime loads instead of capturing.
  Screen-space reflections on the full tier: `PostEffects.ssr`,
  `ssrMaxDistance`, `ssrThickness`, `ssrSteps`, `ssrFade`, `ssrIntensity`;
  `RenderCapabilities.ssr` and `reflectionProbes` (`ssr` is false on
  WebGL2). The prepass's linear depth is `Rg32Float` now, its second
  channel the reflectivity. fx-showcase stations `lod` (2,000 trees:
  1,300,812 triangles with LOD against 5,150,788 at `lodBias` 0) and
  `reflections`.
- `render-targets` (S8), module render: `og.render.renderTarget(world,
  { width, height, format, depth })` makes an `ldr` or `hdr` target whose
  colour is an ordinary `Texture` (a material map, a UI image, a mirror);
  depth is the full tier's (`RenderCapabilities.renderTargetDepth`,
  `maxRenderTargetSize`, `maxCustomPosts`). `og.CameraView` renders a
  second camera into a target or a viewport rect (a monitor, a split
  screen); `og.Mirror` fits an off-axis frustum to its glass. `layers u32`
  (default 1) on `og.Mesh`, `og.Model`, `og.Sprite`, `og.Lines`,
  `og.Points`, `og.Camera`, `og.CameraView` and `og.Mirror`, tested with
  three.js's `&`; `og.render.raycast` reads each mesh's mask; shadows
  take every caster. `type shader` gains `mode full lit|unlit`: vertex
  and fragment entries that the prepass and shadow passes run too. `type
  post` with `og.load.post`, `PostEffects.custom1..4` (four fixed slots),
  `og.render.setPostParam` and `og.render.postErrors` (a pass a device
  fails to compile is skipped and listed). A view draws opaque meshes,
  terrain and sky only, and a frame with views culls on the CPU.
  fx-showcase station `views` (new) and `shaders` (a full-shader dissolve
  and a CRT post pass).
- `render-light` (S7), module render: the og-bake crate (xatlas UVs, a
  BVH, a progressive CPU path tracer, L1 SH light volumes). `og bake
  lightmaps [project] [--samples] [--bounces] [--texels-per-metre]
  [--max-size] [--sky] [--seconds] [--threads] [--scene]` writes
  `<scene>.lightmap.ogt` and `.ktx2` beside the scene, which the cook makes
  an `og.Lightmap` blob (`cook --report` lists it). `og.Model.static`
  marks what the bake covers; the runtime draws a scene's bake when its
  scene and model hashes still match and otherwise reports why through
  `og.render.bakedLighting` (`BakedScene`); `RenderSettings.bakedLighting`
  (on by default) turns it off; `og doctor --project` warns `unbaked`.
  `og.LightVolume` (`size`, `spacing`): baked SH probes sampled as ambient
  light by what moves through them. `og.RectLight` (`color`, `intensity`,
  `width`, `height`, `twoSided`) shaded with LTC. `cookie` on
  `og.SpotLight` (a texture) and `og.PointLight` (a cube), 24 layers at
  most. `Fog.volumetric` with `scattering`, `anisotropy` and
  `volumetricDistance`: a froxel compute pass named `volumetric` on the full tier, a
  sun-only raymarch on WebGL2; `RenderCapabilities.volumetricFog` names
  which. fx-showcase stations `baked` (a Cornell box, `baked:on|off`) and
  `volumetric` (window shafts under a day-night sun, a rect light, a
  lattice cookie, a lantern's cube cookie); gfx scenes `rect-lights`,
  `light-cookies`, `volumetric-fog` and, natively, `baked-cornell`.
- Wave 3's end (`scripts/wave-end.sh`, docs/phase-1.md): the lightmap
  reader builds for wasm32; fx-showcase's light-stage entity is
  `render-light` (its name clashed with the fluids station's `light`
  fluid when one world shows every station) and its `none` look also
  puts the HUD back in the prototype style; tier references recorded
  for WebGL2's sun-only volumetric fog (`volumetric`, `volumetric-fog`),
  the fluid surfaces (`fluids3d` on WebGPU, `water` on WebGL2),
  third-person's `beacon` and the two webgl2 looks whose badge wears the
  preset's style.

Debt round (merged 2026-10-23, in progress: what the depth streams and
`ui/style` left owed, `tests/evidence/debt/<stream>/NOTES.md` each), API
changes by stream:

- `render-targets`: an `og.CameraView` or `og.Mirror` draws what the main
  pass draws on its layers: transparents sorted from the view's eye, world
  sprites and text, particles (`og.ParticleSystem.layers`), lines and
  points; an ldr target or viewport carries exposure and the tone curve,
  an hdr target linear radiance. The main camera keeps GPU culling in a
  frame with views (views and probe faces cull on the CPU and append; the
  GPU cull tests the layer mask). A target's `depth` texture is drawn on
  every tier, WebGL2 included (`RenderCapabilities.renderTargetDepth` is
  true wherever a renderer exists). `PostEffects.custom1` to `custom8`
  (eight slots, `maxCustomPosts` 8 on every tier). `describe` entries name
  the `views` an entity shows in and `Description.views` lists each
  view's own id pass. The mirror's near plane sits a hair off the glass.
  Gfx scenes `views-layers` and `views-depth`.
- `render-lod`: reflection probes are picked per instance (a shared mesh
  splits into one batch per probe); captures draw skinned meshes in their
  pose, terrain, scatter near the box, the sky and transparents; terrain,
  scatter, custom, water and fluid shaders take the probe's ambient and
  reflected light; the prepass reflectivity reads the roughness and
  metalness maps. `expect:entity:<id>:<path><op><value>` and
  `expect:timings:<path><op><value>`; fx-showcase playtests `lod` and
  `reflections`. A probe bake's key hashes the room's content
  (`cooked/probes/<key>.ogt` beside it) and `og doctor --project` reports
  `stale-probe`. Gfx scenes `probe-two-rooms` and `probe-outdoor`.
- `style-names`: og-ui's Rust names say style (`style_file`, `Art`,
  `Theme::art`, an element's `frame`); ui-kit's `style:` command and
  `kit-style-*` sheets; Path Guard and Rumpus Isle on the current cartoon
  art (modest corners, no pills; Path Guard's primary `#974009` for 7:1).
  `og.load.look`'s docs call the look's `ui` line its style.
- `lantern-vale-physics`: the pile's rocks are density 150 (46 kg), light
  enough for the controller's capped push (`pushStrength`, 1,000 N); the
  physics playtest checks the rocks the hero walks into, not the pile's
  own slump. No engine change.
- `geometry-video` (first item): the `debug-overlay` gfx scene draws two
  `og.debug.text` labels. The og-cli doctor tests clear `OG_TSRS` and
  `OG_ENGINE_PACKAGE` themselves and skip an installed tsrs.
- `render-light`: every loaded scene's bake draws (each mesh binds its
  own scene's pages); the physical, array, material-shader and compact
  batches draw lightmaps and light volumes; rect lights light terrain,
  scatter and fluids through LTC and cast shadows (`RectLight.castShadow`,
  a 150° view from the centre read as PCSS); `RenderCapabilities.
  maxLightCookies` (24) with a warning naming the lights left without;
  the WebGL2 raymarch is lit by point, spot and rect lights
  (`RenderCapabilities.volumetricLights`, 8); froxels take eight jittered
  depth samples (no column streaks). Gfx scene `baked-batches`.
- `geometry-video`: the cook builds each model's raycast BVH (`model {
  raycast false }` opts out; `og cook --report` shows `bvhBytes`); tubes
  along an `og.path` space their rings by arc length; timelines hot
  reload; music voices loop (`looping`); per-player video textures; web
  frame capture; video on a mesh's material map and an `og.ui` image.
- `two-d-math`: the DragonBones import carries skew, colour offsets,
  zOrder, IK timelines, nested armatures, bounding boxes
  (`og.anim.skeleton2dBoxes`) and Bézier curves; IK over chains of any
  length (one and two bones keep their exact solvers, longer chains
  FABRIK). Behaviour change: `og.IkTarget2D.weight` multiplies the
  constraint's own weight; it used to replace it. The game-2d template's
  lamp keeper walks (`keeper:<x>`, feet planted by IK).
- `probe-rooms`: trace spans carry their real starts (systems, passes,
  og-lanes jobs on their lane threads, cook outputs); GPU pass timing
  through timestamp queries (`ProbeTimings.gpuPasses`, the trace's gpu
  thread, `RenderCapabilities.gpuTimestamps`; every pass writes its own
  pair, resolved once the frame is done, so Metal reads real numbers);
  the web trace (the shell's `trace()`/`saveTrace()`, the play page's
  `?trace`); scene write-back to each instantiated scene's own `.ogt`
  with spawned entities under a marked section (`resource <Name> { }`
  scene entries); opaque UI backgrounds count in `describe`'s occlusion;
  `cook --report` figures for every asset type with new budgets
  (`soundSeconds`, `videoSeconds`, `fontGlyphs`, `navPolygons`,
  `sceneEntities`, `emitterParticles`); room playtests take `then` and
  `quit` (a lockstep gate's restore no longer moves scheduled steps);
  `?room=<id>` Play links (`RoomInfo.link`); the arena archetype polished
  to taste 100 / feel 66.

## 0.2.0 (2026-10-23)

API additions and changes (a minor bump, §3), by the stream that landed
each:

- API: every word of the kit HUDs is a field with its English default
  (kit-words, 2026-10-22): `PlatformerHudDesc.jumpLabel`,
  `ShooterHudDesc.killsLabel` and `fireLabel`, `PuzzleHudDesc.movesLabel`
  and `targetLabel` (`{target}`), `IdleHudDesc.levelLabel` (`{level}`) and
  `rateLabel` (`{rate}`), and `pauseLabel` on each of them (the pause
  button's name for a screen reader); a localised game passes its string
  table's (`og.i18n.t`). Additive; the defaults are the words the HUDs
  showed before.

The `platformer-2d` stream (Brasswick's runtime findings,
`examples/brasswick/FINDINGS.md`, 2026-10-22):

- `og.WorldRestored` (event, module world): delivered the tick after a
  `world.restore` applies, with `from` (the tick the world was at) and
  `tick` (the saved tick), for systems that reset what lives in the session
  (#8). `world.restore`'s doc says so, and that an `og run --commands`
  script goes on to its last step whichever way a restore moved og.Time.
- `og.PARTICLES_ARMED` (const, module render): `ParticleSystem.startTick`'s
  default, never written; a one-shot (`looping false`) emitter stays armed
  and emits nothing until a tick is written, a looping one emits as if
  started at tick 0 (#3). `startTick` carries the default.
- `og.CharacterController.pushStrength` and
  `og.PlatformerController.pushBodies` (default true) and `pushStrength`
  (newtons, default 1000, tunable): the push on the dynamic bodies a
  controller walks or runs into is an impulse capped by the strength, so a
  light body is shoved and a heavy one holds (#6). Unit `newton` added.
- `og.TilemapLayer.colliderLayers` and `colliderMask`: a collision layer's
  own `og.Collider` layers and mask, from its `.ogt` line or Tiled int
  property; 0 takes `Tilemap.colliderLayers` and every layer (#5).
- `PlatformAnswer.time`: a slot call (`saveSlot`, `loadSlot`) answers the
  time of the store answer that ended it (#7). `CharacterEvent`'s doc says
  the event is read the tick after the move (#14).

Also:

- `og run --commands`: the script ends when its last step has run, wherever
  a restore moved og.Time (#13).
- `create-game --profile 2d` writes the 2D starter `templates/game-2d`
  (shipped as `template-2d/`), as does a premise that says 2D, platformer,
  side-scroller, metroidvania or pixel art without saying 3D (#1). The
  install test scaffolds and tests it.
- Knowledge: og-game §8 gives the flipbook grammar line by line (#2);
  og-feel §12 reads the jump event the tick after (#14).

The `dogfood-7` stream (Rumpus Isle, a local-multiplayer party game,
`examples/rumpus-isle/FINDINGS.md`, 2026-10-22):

- `og test --filter <path>`: selects suite files by a substring of their
  project-relative path before anything runs; an empty selection fails
  (#14). No API change.
- Knowledge: og-build on headless rendering (`OG_RENDERER=none`, the
  lavapipe fallback) (#7); og-ui on the UI's entrance timing and the
  presses it rejects (#10) and the iris midpoint (#12); og-game on the
  post-solver velocity a contact event carries (#9).

## 0.1.1 (2026-10-22)

- Fix: the published template, archetypes and knowledge scaffold carried no
  `.gitignore` (npm drops the file from every tarball), so a scaffolded game
  and the platform's overlay had none. Stored as `gitignore` in the packages
  and written back as `.gitignore` by `create-game` and the knowledge mount,
  as `gitattributes` already was. The install test checks both files.
- No API change.

## 0.1.0 (2026-10-22)

The first published version: the API as `crates/og-api/api.ogt` stands at
the tag (modules math, reflect, world, system, game, transform, time, rng,
input, camera, render, load, physics, atmosphere, anim, tween, tilemap,
audio, nav, ai, platform, i18n, ui, systems, commands, probe, terrain,
stream), the `og` tool (run, dev, test, polish, build, cook, probe, check,
sfx, skin, doctor, api), the project template and eight archetypes, the
ten `og-*` skills and the generated wiki. Everything before it is the
build-out recorded in `docs/phase-1.md`.
