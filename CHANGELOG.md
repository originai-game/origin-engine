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
