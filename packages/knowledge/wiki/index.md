<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The Origin Engine API

`@origingame/engine`, API version 1: 36 modules, generated from `crates/og-api/api.ogt`. Each module has one page; every signature on it is quoted from the IDL.

The Origin Engine API. Game code declares components and systems, and reads the world through typed views over engine memory.

Every failure is a returned Result; desc objects may omit any field; nothing reads the wall clock inside a tick.

## Modules

Grouped by the skill that teaches them: the skill says how much of each to use and how to prove it, the module's page is every signature.

### `og-core`

| Module | TypeScript | Publish builds | Summary |
|---|---|---|---|
| [`math`](math.md) | types and tokens | kept | Plain math values in the three.js vocabulary. They are data: the engine owns the operations, so results are identical on every host. |
| [`reflect`](reflect.md) | named exports | kept | Reflection: components and resources are declared once with their field layout and metadata, and registered by name. |
| [`world`](world.md) | types and tokens | kept | The world: an archetype ECS. Entities are generational handles; components are columns in engine memory. |
| [`system`](system.md) | named exports | kept | Systems: functions the engine calls once per tick with their matched archetypes. |
| [`rng`](rng.md) | types and tokens | kept | Deterministic randomness. Every random draw comes from an Rng stream; the sequence for a seed is identical on every host and in the Web scaffold's rng.ts. |
| [`procgen`](procgen.md) | `og.procgen.*` | kept | Procedural generation: gradient noise (Perlin, simplex, FBM), Poisson-disk points, BSP dungeons and wave function collapse, all deterministic. Each generator takes a `seed`; give it the seed of a fork of og.Rng, `world.resource(og.Rng)`'s value `.fork("dungeon").seed` (a fork needs og.Rng among a system's reads only), so the same world and label give the same output and the main stream does not move. A generator draws only from a stream on that seed, in a fixed order, with arithmetic that is bit-identical on every host: the same seed gives the same map natively and on the web, in a replay and after a restore. |

### `og-game`

| Module | TypeScript | Publish builds | Summary |
|---|---|---|---|
| [`game`](game.md) | `og.*` | kept | The game entry: what the bundle's default export declares. |
| [`transform`](transform.md) | `og.transform.*` | kept | Transforms in the three.js vocabulary: position, quaternion and scale, composed into matrixWorld down the parent chain. |
| [`input`](input.md) | `og.input.*` | kept | Input as the game sees it: the Input resource for polling, events for edges. |
| [`load`](load.md) | `og.load.*` | kept | Asset loading from cooked output: `og cook` turns assets/ into cooked/ (.ogb blobs and the manifest cooked/.cook.ogt), and the runtime reads only cooked/. |
| [`stream`](stream.md) | `og.stream.*` | kept | Asset streaming groups (phase 3): a scene's `stream "<name>" { asset ...; distance x y z r [unload d] \| volume "<entity>"; priority n }` blocks name glTF models (without skins or morph targets), textures, texture arrays, flipbooks (an `array true` sheet brings its array), sounds, environment maps, palettes, looks and colour LUTs (`.cube`) that load and unload together, and what loads them: the camera within `r` of the anchor (kept until it is past `unload`, 1.25 r by default), the camera inside the named entity's box (its position ± half its scale, turned with it), or neither (og.stream.request only). `og cook` packs each group's files into `cooked/stream/<name>.ogb`; `og build --publish` ships the packs beside the boot bundle, which the shell and the service worker fetch first, and fetches a pack only when its group loads. |

### `og-feel`

| Module | TypeScript | Publish builds | Summary |
|---|---|---|---|
| [`time`](time.md) | `og.time.*` | kept | Simulation time. Game code never reads the wall clock; it reads Time. |
| [`camera`](camera.md) | `og.camera.*` | kept | The main camera, as a resource with three.js PerspectiveCamera and OrthographicCamera parameter names. |
| [`tween`](tween.md) | `og.tween.*` | kept | Tweens: animate any numeric field of a component (og.Transform.position, og.Sprite.color, og.Sprite.opacity, a game's own f32) from its value to a target over time with an easing curve, a delay, loops and yoyo; chain them in sequences or run them in parallel. Advanced by the engine's og.tween system after the game's systems and before og.anim and physics, by scaled time (slow motion, hit stop) unless unscaled. |
| [`systems`](systems.md) | types and tokens | kept | Systems the engine ships as TypeScript: game-level code, listed explicitly in GameDesc.systems like the game's own, so the schedule (and dump_world) shows them and nothing runs that the game did not ask for. |

### `og-render`

| Module | TypeScript | Publish builds | Summary |
|---|---|---|---|
| [`render`](render.md) | `og.render.*` | kept | What is drawn: meshes with materials. Draws are instanced by (geometry, material) automatically; there is no per-object draw call. |
| [`atmosphere`](atmosphere.md) | `og.atmosphere.*` | kept | Fog, a procedural sky, the day-night cycle, weather, the wind that sways vegetation and water, declared as resources and components (the E5 round). Everything is off by default: a world that sets none of them draws exactly as before. |
| [`terrain`](terrain.md) | `og.terrain.*` | kept | Terrain (phase 2): a height field over a rectangle, cooked by `og cook` from a `type terrain` .ogt (a 16-bit grey PNG or an EXR heightmap, an optional control map, up to four layered materials with slope and altitude rules, their maps separate textures or layers of two texture arrays: `layers` and `layerNormals` with each layer's `albedoLayer` and `normalLayer`, two sampled textures instead of up to eight) into tiles; drawn as a geometry clipmap around the camera (nested levels by distance, no cracks) that casts and receives shadows under the fog; collided with as a height field; baked into navmeshes by a navmesh file's `terrain` entry. |
| [`video`](video.md) | types and tokens | kept | Footage in a game (docs/design.md §2.9): a `type video` asset cooked by ffmpeg into a texture array of its frames and its sound, loaded with og.load.video and played by og.VideoPlayer. The frame shown is a Texture (VideoInfo.texture, also VideoPlayer.texture) any material map, og.Sprite or UI image draws, and the sound plays on the entity's og.AudioSource in step with it. |
| [`debug`](debug.md) | `og.debug.*` | dropped | Debug draw: lines, boxes, spheres, arrows, axes and labels in the world, each for a number of ticks, drawn over the finished frame (after every post effect, before the UI) so they read in every capture and never hide behind geometry; and the debug views (physics, navigation, bounds, levels of detail, and the lit image's terms: shadow cascades, ambient occlusion, albedo, image-based and direct light) the command channel's `debug:` actions switch. |

### `og-model`

| Module | TypeScript | Publish builds | Summary |
|---|---|---|---|
| [`shapes`](shapes.md) | types and tokens | kept | Two-dimensional outlines in three.js's words: og.Shape and og.Path2 are three.js's Shape and Path, built by the same calls (moveTo, lineTo, quadraticCurveTo, bezierCurveTo, splineThru, arc, absarc, ellipse, absellipse, closePath) and flattened to points by getPoints, so a three.js outline is a paste. og.render.shapeGeometry, extrudeGeometry and sweepGeometry take them; only their points cross to the engine. |

### `og-physics`

| Module | TypeScript | Publish builds | Summary |
|---|---|---|---|
| [`physics`](physics.md) | `og.physics.*` | kept | Rigid bodies over rapier (rapier3d in the 3D profile, rapier2d in 2D, both with enhanced determinism): game code owns the components, the engine's og.physics system steps them after the game's systems and writes the dynamic bodies back. |

### `og-anim`

| Module | TypeScript | Publish builds | Summary |
|---|---|---|---|
| [`anim`](anim.md) | `og.anim.*` | kept | Skeletal animation over og-anim: glTF skins and clips (cooked into the `<model>#anim` blob, loaded with og.load.animation), blend-tree graphs with layered state machines, root motion, two-bone and FABRIK IK and blendshapes, evaluated by the engine's og.anim system after the game's systems and before physics. |
| [`path`](path.md) | `og.path.*` | kept | Paths: a curve through or controlled by points, measured by arc length so a parameter t from 0 to 1 moves along it at an even speed however its points are spaced. catmullRom passes through every point (three.js CatmullRomCurve3, tension 0.5), bezier chains cubic segments `p0 c0 c1 p1 c2 c3 p2 ...` (three.js CubicBezierCurve3 in a CurvePath), linear joins the points with straight segments. A closed path joins its last point to its first and wraps t. |
| [`timeline`](timeline.md) | `og.timeline.*` | kept | Timelines: cutscenes and sequences as data, Unity's Timeline and Godot's AnimationPlayer as an `.ogt` the agent reads and diffs. A `type timeline` file has a `duration` in seconds, `path "<name>" { ... }` blocks (as og.path's) and `track <kind> ["<entity>"] { key <seconds> { ... } }` blocks; `og cook` cooks it (importing caption files) and og.load.timeline loads it. Tracks: `camera` (position and target keys, or `rail "<path>"` with `at` 0..1 along it, or `follow "<entity>"` with offsets; fov; eased), `transform "<entity>"` (position, rotation in degrees or quaternion, scale; eased), `animation "<entity>"` (clip, speed, blendIn, loop; `model` names the glTF whose clips they are, else the entity's og.Model), `audio` (sound, volume, pitch, bus), `music` (sound, volume, fadeIn, fadeOut, end, loop, bus), `caption` (text from `key` to `end` with a UI type step `style` and an `anchor`; `captions "narration.srt" [offset]` imports a SubRip file or word timestamps `.json`), `event` (`do "<command-channel action>"`: teleport, set, a game's registered command; `retroactive true` runs it when a seek passes it), `transition` (kind cut, fade, wipe, dissolve, iris or slide; mode through, in or out; duration, color, from, over og.ui's screen cover) and `marker` (`name`, sending og.TimelineMarker when played past). |

### `og-2d`

| Module | TypeScript | Publish builds | Summary |
|---|---|---|---|
| [`tilemap`](tilemap.md) | `og.tilemap.*` | kept | Tilemaps: layers of tiles from a tileset atlas (an `.ogt` of `type tileset`: grid, solid tiles and collision rects, animated tiles, 47-blob and 16-bitmask auto-tiling terrains), loaded from a cooked map (a hand-written `.ogt` of `type tilemap`, a Tiled `.tmj` or an LDtk `.ldtk`) with og.load.tilemap and placed by an og.Tilemap component. |

### `og-audio`

| Module | TypeScript | Publish builds | Summary |
|---|---|---|---|
| [`audio`](audio.md) | `og.audio.*` | kept | Sound: inside the tick, the engine's og.audio system (after the game's systems and physics) turns AudioSource changes into a deterministic, time-stamped command stream; outside the tick the host's mixer renders it (an AudioWorklet on the web, the output device, a capture or a null sink natively). |

### `og-ai`

| Module | TypeScript | Publish builds | Summary |
|---|---|---|---|
| [`nav`](nav.md) | `og.nav.*` | kept | Navigation: grids (A* and jump point search with eight-way moves that never cut a blocked corner, line-of-sight smoothing), flow fields for crowds, and navmeshes (baked at cook time from a level's static colliders and models, or built at runtime for a flat procedural level) with funnel paths and box obstacles that rebuild only the tiles they touch. |
| [`ai`](ai.md) | `og.ai.*` | kept | Enemy and NPC AI as data: behaviour trees and state machines in `.ogt` files of `type behaviour` (og-ai §3 has every line), perception (sight cones checked against physics occlusion, hearing of spatial sounds and game noises) and a blackboard per agent. |

### `og-platform-sdk`

| Module | TypeScript | Publish builds | Summary |
|---|---|---|---|
| [`platform`](platform.md) | `og.platform.*` | kept | Platform services as host capabilities (docs/design.md §2.8): cloud store, leaderboards, telemetry, Origin Point and AI marks. Game code names a method and its arguments, never a credential, a URL or a cookie: the host owns them. On the web the shell forwards each call to the Player through origin-checked postMessage (Player protocol 2, docs/web-export.md); natively the host answers from a local JSON store (`og run --platform-store`, default `<project>/.og/platform.json`). |

### `og-i18n`

| Module | TypeScript | Publish builds | Summary |
|---|---|---|---|
| [`i18n`](i18n.md) | `og.i18n.*` | kept | Localization: one string table per locale (`.ogt` of `type strings`: `locale "ja"`, its font chain `fonts "..."`, `string "key" "message"`), messages in a subset of ICU MessageFormat (`{name}`, `{n, number}`, `{n, plural, =0 {..} one {..} other {..}}` with `#`, `{g, select, a {..} other {..}}`), CLDR plural rules, and the locale the host asks for (the browser's `navigator.language` on the web, `og run --locale` natively). |

### `og-ui`

| Module | TypeScript | Publish builds | Summary |
|---|---|---|---|
| [`ui`](ui.md) | `og.ui.*` | kept | The engine's UI: an immediate-mode toolkit. Systems describe the tree every tick with these calls; the engine lays it out Clay-style, styles it with the game's UI style (the engine's default style, for prototypes only, until the game picks one), draws it over the frame and answers the probe (dump_ui, touch_move_to, move_to_text) from it. |

### `og-build`

| Module | TypeScript | Publish builds | Summary |
|---|---|---|---|
| [`commands`](commands.md) | `og.commands.*` | dropped | The command channel: actions scheduled on simulation time, from `og run --commands "t:action[:args];..."` or the probe's commands(). |
| [`probe`](probe.md) | types and tokens | kept | The probe: RuntimeProbe protocol 5 (origin/packages/contracts/src/probe.ts) served natively, plus the engine's wire members. |

### `og-net`

| Module | TypeScript | Publish builds | Summary |
|---|---|---|---|
| [`net`](net.md) | `og.net.*` | kept | Deterministic lockstep over a room (docs/design.md §2.10): every peer runs the whole world, and what crosses the room is each player's input, one per tick, and the world's hash. Turn it on with og.Lockstep (`enabled`, `players`); join a room (og.platform.roomJoin, or `og run --room ws://127.0.0.1:P/<room>` with `og relay` natively). Until the room holds `players` peers the world runs freely on each (a lobby, the local input driving the local player); then the authority (the lowest peer number) starts: every other peer takes its world's snapshot, the seats go to the peers in number order, and from then on tick t runs only when every seated player's input for t is in. A peer that joins a running room takes a free seat (or watches) from a snapshot a running peer hands it. |

### `og-taste`

| Module | TypeScript | Publish builds | Summary |
|---|---|---|---|
| [`art`](art.md) | types and tokens | kept | A game's art contract: what every asset of its style holds, as seven resources a look sets beside the render ones (docs/art.md §2.1): the palette its texels sit on (ArtPalette), one pixel size for 2D art (ArtPixels), one texel density for 3D surfaces (ArtTexels), the shape language of its models (ArtShape), its line (ArtLine), the step of its motion (ArtMotion) and its camera (ArtCamera). A style is the assets, not a filter on the frame: the look's render resources treat the frame, these say what the assets themselves must be. |

## Units

Field units the docs and Tune use.

| Unit | Means |
|---|---|
| `m` | metres |
| `s` | seconds of simulation time |
| `ms` | milliseconds |
| `rad` | radians |
| `deg` | degrees |
| `hz` | ticks or cycles per second |
| `ticks` | fixed simulation steps |
| `cssPx` | CSS pixels of the play surface, origin at the top left |
| `ratio` | a fraction between 0 and 1 |
| `bytes` | bytes |
| `kg` | kilograms |
| `newton` | newtons: the force that gives a kilogram one metre per second squared |
| `newtonSecond` | newton-seconds: an impulse, the momentum that changes a kilogram's velocity by one metre per second |
| `newtonMetre` | newton-metres: a torque, a newton pushing one metre from the axis it turns about |
| `mps` | metres per second |
| `mps2` | metres per second squared |
| `radps` | radians per second |
| `rpm` | revolutions per minute |
| `perSecond` | a rate per second: the fraction of a value that decays in one second |
| `kgPerM3` | kilograms per cubic metre (per square metre in the 2D profile) |
| `db` | decibels |
| `evPerSecond` | exposure stops (EV) per second of simulation time |
| `bpm` | beats per minute |

## Every name

2002 entries: modules' functions, types, records, column kinds and constants, and every interface member and resource method.

| Name | Kind | Page | Binding symbol |
|---|---|---|---|
| [`A11yActionKind`](input.md#type-A11yActionKind) | enum | input | — |
| [`A11yNode`](ui.md#type-A11yNode) | struct | ui | — |
| [`A11yRequest`](input.md#type-A11yRequest) | struct | input | — |
| [`A11yTree`](ui.md#type-A11yTree) | struct | ui | — |
| [`Ability`](ui.md#type-Ability) | desc struct | ui | — |
| [`AbilityBarDesc`](ui.md#type-AbilityBarDesc) | desc struct | ui | — |
| [`AbilityBarResult`](ui.md#type-AbilityBarResult) | struct | ui | — |
| [`AbilitySlotDesc`](ui.md#type-AbilitySlotDesc) | desc struct | ui | — |
| [`AccordionDesc`](ui.md#type-AccordionDesc) | desc struct | ui | — |
| [`AccordionResult`](ui.md#type-AccordionResult) | struct | ui | — |
| [`AccordionSection`](ui.md#type-AccordionSection) | desc struct | ui | — |
| [`ActionButtonDesc`](ui.md#type-ActionButtonDesc) | desc struct | ui | — |
| [`ActionDesc`](input.md#type-ActionDesc) | desc struct | input | — |
| [`ActionHint`](ui.md#type-ActionHint) | desc struct | ui | — |
| [`ActionHintsDesc`](ui.md#type-ActionHintsDesc) | desc struct | ui | — |
| [`ActionHintsResult`](ui.md#type-ActionHintsResult) | struct | ui | — |
| [`ActionInfo`](input.md#type-ActionInfo) | struct | input | — |
| [`ActionKind`](input.md#type-ActionKind) | enum | input | — |
| [`ActionState`](input.md#type-ActionState) | struct | input | — |
| [`AdvanceOperation`](probe.md#type-AdvanceOperation) | struct | probe | — |
| [`AI_FAILURE`](ai.md#const-AI_FAILURE) | constant | ai | — |
| [`AI_INTERRUPTED`](ai.md#const-AI_INTERRUPTED) | constant | ai | — |
| [`AI_RUNNING`](ai.md#const-AI_RUNNING) | constant | ai | — |
| [`AI_SUCCESS`](ai.md#const-AI_SUCCESS) | constant | ai | — |
| [`AiDescription`](ai.md#type-AiDescription) | struct | ai | — |
| [`AiMarkDesc`](platform.md#type-AiMarkDesc) | desc struct | platform | — |
| [`AlertDesc`](ui.md#type-AlertDesc) | desc struct | ui | — |
| [`Align`](ui.md#type-Align) | enum | ui | — |
| [`AmmoDesc`](ui.md#type-AmmoDesc) | desc struct | ui | — |
| [`Anchor`](ui.md#type-Anchor) | enum | ui | — |
| [`AnchorDesc`](ui.md#type-AnchorDesc) | desc struct | ui | — |
| [`AnchorInfo`](ui.md#type-AnchorInfo) | struct | ui | — |
| [`AnchorScaleMode`](ui.md#type-AnchorScaleMode) | enum | ui | — |
| [`Animation`](anim.md#type-Animation) | struct | anim | — |
| [`Antialias`](render.md#type-Antialias) | enum | render | — |
| [`AoMethod`](render.md#type-AoMethod) | enum | render | — |
| [`AoMethod()`](render.md#column-AoMethod) | column kind | render | — |
| [`AoResolution`](render.md#type-AoResolution) | enum | render | — |
| [`AoResolution()`](render.md#column-AoResolution) | column kind | render | — |
| [`AoResolutionSetting`](render.md#type-AoResolutionSetting) | enum | render | — |
| [`AoResolutionSetting()`](render.md#column-AoResolutionSetting) | column kind | render | — |
| [`ArchetypeDump`](probe.md#type-ArchetypeDump) | struct | probe | — |
| [`ArchetypeView`](world.md#type-ArchetypeView) | interface | world | — |
| [`ArchetypeView.column`](world.md#member-ArchetypeView-column) | method | world | `og_archetype_view_column` |
| [`ArchetypeView.count`](world.md#member-ArchetypeView-count) | property | world | `og_archetype_view_get_count` |
| [`ArchetypeView.entities`](world.md#member-ArchetypeView-entities) | property | world | `og_archetype_view_get_entities` |
| [`ArchetypeView.entity`](world.md#member-ArchetypeView-entity) | method | world | `og_archetype_view_entity` |
| [`ArchetypeView.has`](world.md#member-ArchetypeView-has) | method | world | `og_archetype_view_has` |
| [`ArchetypeView.rows`](world.md#member-ArchetypeView-rows) | property | world | `og_archetype_view_get_rows` |
| [`ArchetypeView.tick`](world.md#member-ArchetypeView-tick) | property | world | `og_archetype_view_get_tick` |
| [`ArtBrush`](art.md#type-ArtBrush) | enum | art | — |
| [`ArtBrush()`](art.md#column-ArtBrush) | column kind | art | — |
| [`ArtFilter`](art.md#type-ArtFilter) | enum | art | — |
| [`ArtFilter()`](art.md#column-ArtFilter) | column kind | art | — |
| [`ArtFit`](ui.md#type-ArtFit) | enum | ui | — |
| [`ArtLinePlace`](art.md#type-ArtLinePlace) | enum | art | — |
| [`ArtLinePlace()`](art.md#column-ArtLinePlace) | column kind | art | — |
| [`ArtMask`](ui.md#type-ArtMask) | enum | ui | — |
| [`ArtMips`](art.md#type-ArtMips) | enum | art | — |
| [`ArtMips()`](art.md#column-ArtMips) | column kind | art | — |
| [`ArtNormals`](art.md#type-ArtNormals) | enum | art | — |
| [`ArtNormals()`](art.md#column-ArtNormals) | column kind | art | — |
| [`ArtProjection`](art.md#type-ArtProjection) | enum | art | — |
| [`ArtProjection()`](art.md#column-ArtProjection) | column kind | art | — |
| [`ArtRotation`](art.md#type-ArtRotation) | enum | art | — |
| [`ArtRotation()`](art.md#column-ArtRotation) | column kind | art | — |
| [`ArtSlotDesc`](ui.md#type-ArtSlotDesc) | desc struct | ui | — |
| [`Asset`](load.md#type-Asset) | handle | load | — |
| [`Asset()`](load.md#column-Asset) | column kind | load | — |
| [`AudioBus`](audio.md#type-AudioBus) | enum | audio | — |
| [`AudioBus()`](audio.md#column-AudioBus) | column kind | audio | — |
| [`AudioEventOrder`](audio.md#type-AudioEventOrder) | enum | audio | — |
| [`AudioEventOrder()`](audio.md#column-AudioEventOrder) | column kind | audio | — |
| [`AudioResampler`](audio.md#type-AudioResampler) | enum | audio | — |
| [`AudioResampler()`](audio.md#column-AudioResampler) | column kind | audio | — |
| [`BackdropEffect`](ui.md#type-BackdropEffect) | desc struct | ui | — |
| [`BadgeKind`](ui.md#type-BadgeKind) | enum | ui | — |
| [`BakedScene`](render.md#type-BakedScene) | struct | render | — |
| [`BannerDesc`](ui.md#type-BannerDesc) | desc struct | ui | — |
| [`BannerKind`](ui.md#type-BannerKind) | enum | ui | — |
| [`BannerResult`](ui.md#type-BannerResult) | struct | ui | — |
| [`Behaviour`](ai.md#type-Behaviour) | handle | ai | — |
| [`Behaviour()`](ai.md#column-Behaviour) | column kind | ai | — |
| [`BehaviourInfo`](ai.md#type-BehaviourInfo) | struct | ai | — |
| [`BezierKey`](ui.md#type-BezierKey) | struct | ui | — |
| [`BindingConflict`](input.md#type-BindingConflict) | struct | input | — |
| [`BlendMode`](ui.md#type-BlendMode) | enum | ui | — |
| [`BlendshapeSource`](anim.md#type-BlendshapeSource) | enum | anim | — |
| [`BlendshapeSource()`](anim.md#column-BlendshapeSource) | column kind | anim | — |
| [`BlurEffect`](ui.md#type-BlurEffect) | desc struct | ui | — |
| [`BODY_DYNAMIC`](physics.md#const-BODY_DYNAMIC) | constant | physics | — |
| [`BODY_KINEMATIC`](physics.md#const-BODY_KINEMATIC) | constant | physics | — |
| [`BODY_STATIC`](physics.md#const-BODY_STATIC) | constant | physics | — |
| [`BonePose`](physics.md#type-BonePose) | struct | physics | — |
| [`bool()`](reflect.md#column-bool) | column kind | reflect | — |
| [`Box3`](math.md#type-Box3) | struct | math | — |
| [`box3`](math.md#value-box3) | value | math | — |
| [`Box3Math`](math.md#type-Box3Math) | interface | math | — |
| [`Box3Math.applyMatrix4`](math.md#member-Box3Math-applyMatrix4) | method | math | — |
| [`Box3Math.containsBox`](math.md#member-Box3Math-containsBox) | method | math | — |
| [`Box3Math.containsPoint`](math.md#member-Box3Math-containsPoint) | method | math | — |
| [`Box3Math.empty`](math.md#member-Box3Math-empty) | method | math | — |
| [`Box3Math.expandByPoint`](math.md#member-Box3Math-expandByPoint) | method | math | — |
| [`Box3Math.fromPoints`](math.md#member-Box3Math-fromPoints) | method | math | — |
| [`Box3Math.getCenter`](math.md#member-Box3Math-getCenter) | method | math | — |
| [`Box3Math.getSize`](math.md#member-Box3Math-getSize) | method | math | — |
| [`Box3Math.intersectsBox`](math.md#member-Box3Math-intersectsBox) | method | math | — |
| [`Box3Math.intersectsSphere`](math.md#member-Box3Math-intersectsSphere) | method | math | — |
| [`Box3Math.isEmpty`](math.md#member-Box3Math-isEmpty) | method | math | — |
| [`Box3Math.union`](math.md#member-Box3Math-union) | method | math | — |
| [`BoxGeometryDesc`](render.md#type-BoxGeometryDesc) | desc struct | render | — |
| [`BreadcrumbsDesc`](ui.md#type-BreadcrumbsDesc) | desc struct | ui | — |
| [`BreadcrumbsResult`](ui.md#type-BreadcrumbsResult) | struct | ui | — |
| [`Breakpoint`](ui.md#type-Breakpoint) | enum | ui | — |
| [`BspDesc`](procgen.md#type-BspDesc) | desc struct | procgen | — |
| [`BspResult`](procgen.md#type-BspResult) | struct | procgen | — |
| [`BufferGeometryDesc`](render.md#type-BufferGeometryDesc) | desc struct | render | — |
| [`BuildProfile`](probe.md#type-BuildProfile) | enum | probe | — |
| [`BundleInfo`](load.md#type-BundleInfo) | struct | load | — |
| [`BurstDesc`](ui.md#type-BurstDesc) | desc struct | ui | — |
| [`BurstEvent`](ui.md#type-BurstEvent) | struct | ui | — |
| [`ButtonDesc`](ui.md#type-ButtonDesc) | desc struct | ui | — |
| [`ButtonVariant`](ui.md#type-ButtonVariant) | enum | ui | — |
| [`CalendarDesc`](ui.md#type-CalendarDesc) | desc struct | ui | — |
| [`CalendarMark`](ui.md#type-CalendarMark) | desc struct | ui | — |
| [`CalendarResult`](ui.md#type-CalendarResult) | struct | ui | — |
| [`CallbackEvent`](ui.md#type-CallbackEvent) | struct | ui | — |
| [`Camera.lookAt`](camera.md#member-Camera-lookAt) | method | camera | `og_camera_look_at` |
| [`Camera.screenToRay`](camera.md#member-Camera-screenToRay) | method | camera | `og_camera_screen_to_ray` |
| [`Camera.worldToScreen`](camera.md#member-Camera-worldToScreen) | method | camera | `og_camera_world_to_screen` |
| [`CapsuleGeometryDesc`](render.md#type-CapsuleGeometryDesc) | desc struct | render | — |
| [`Capture`](probe.md#type-Capture) | struct | probe | — |
| [`CaptureAccept`](input.md#type-CaptureAccept) | enum | input | — |
| [`CaptureDesc`](input.md#type-CaptureDesc) | desc struct | input | — |
| [`CaptureState`](input.md#type-CaptureState) | struct | input | — |
| [`CardDesc`](ui.md#type-CardDesc) | desc struct | ui | — |
| [`CardMedia`](ui.md#type-CardMedia) | desc struct | ui | — |
| [`CardVariant`](ui.md#type-CardVariant) | enum | ui | — |
| [`CarouselDesc`](ui.md#type-CarouselDesc) | desc struct | ui | — |
| [`CarouselResult`](ui.md#type-CarouselResult) | struct | ui | — |
| [`CastBarDesc`](ui.md#type-CastBarDesc) | desc struct | ui | — |
| [`CastState`](ui.md#type-CastState) | enum | ui | — |
| [`CelebrationDesc`](ui.md#type-CelebrationDesc) | desc struct | ui | — |
| [`CelebrationKind`](ui.md#type-CelebrationKind) | enum | ui | — |
| [`CellRect`](procgen.md#type-CellRect) | struct | procgen | — |
| [`CHARACTER_AIR_JUMPED`](physics.md#const-CHARACTER_AIR_JUMPED) | constant | physics | — |
| [`CHARACTER_BUMPED`](physics.md#const-CHARACTER_BUMPED) | constant | physics | — |
| [`CHARACTER_JUMPED`](physics.md#const-CHARACTER_JUMPED) | constant | physics | — |
| [`CHARACTER_LANDED`](physics.md#const-CHARACTER_LANDED) | constant | physics | — |
| [`CHARACTER_LEFT_GROUND`](physics.md#const-CHARACTER_LEFT_GROUND) | constant | physics | — |
| [`CHARACTER_WALL_JUMPED`](physics.md#const-CHARACTER_WALL_JUMPED) | constant | physics | — |
| [`ChartDesc`](ui.md#type-ChartDesc) | desc struct | ui | — |
| [`ChartKind`](ui.md#type-ChartKind) | enum | ui | — |
| [`ChartResult`](ui.md#type-ChartResult) | struct | ui | — |
| [`ChartSeries`](ui.md#type-ChartSeries) | desc struct | ui | — |
| [`CheckFieldDesc`](ui.md#type-CheckFieldDesc) | desc struct | ui | — |
| [`CheckListResult`](ui.md#type-CheckListResult) | struct | ui | — |
| [`ChipDesc`](ui.md#type-ChipDesc) | desc struct | ui | — |
| [`ChoiceIndexResult`](ui.md#type-ChoiceIndexResult) | struct | ui | — |
| [`ChoiceResult`](ui.md#type-ChoiceResult) | struct | ui | — |
| [`Choreo`](ui.md#type-Choreo) | union | ui | — |
| [`ChoreoCascade`](ui.md#type-ChoreoCascade) | struct | ui | — |
| [`ChoreoPlay`](ui.md#type-ChoreoPlay) | struct | ui | — |
| [`ChoreoStagger`](ui.md#type-ChoreoStagger) | struct | ui | — |
| [`ChoreoSteps`](ui.md#type-ChoreoSteps) | struct | ui | — |
| [`CircleGeometryDesc`](render.md#type-CircleGeometryDesc) | desc struct | render | — |
| [`ClipAsset`](anim.md#type-ClipAsset) | handle | anim | — |
| [`ClipAsset()`](anim.md#column-ClipAsset) | column kind | anim | — |
| [`ClipInfo`](anim.md#type-ClipInfo) | struct | anim | — |
| [`ClipMode`](ui.md#type-ClipMode) | enum | ui | — |
| [`Color`](math.md#type-Color) | struct | math | — |
| [`color`](math.md#value-color) | value | math | — |
| [`Color()`](reflect.md#column-Color) | column kind | reflect | — |
| [`ColorMath`](math.md#type-ColorMath) | interface | math | — |
| [`ColorMath.fromHex`](math.md#member-ColorMath-fromHex) | method | math | — |
| [`ColorMath.fromHsl`](math.md#member-ColorMath-fromHsl) | method | math | — |
| [`ColorMath.lerp`](math.md#member-ColorMath-lerp) | method | math | — |
| [`ColorMath.toHex`](math.md#member-ColorMath-toHex) | method | math | — |
| [`ColorPickerDesc`](ui.md#type-ColorPickerDesc) | desc struct | ui | — |
| [`ColorPickerResult`](ui.md#type-ColorPickerResult) | struct | ui | — |
| [`ComboDesc`](ui.md#type-ComboDesc) | desc struct | ui | — |
| [`CommandDesc`](commands.md#type-CommandDesc) | desc struct | commands | — |
| [`CommandHandler`](commands.md#type-CommandHandler) | callback | commands | — |
| [`CommandInfo`](commands.md#type-CommandInfo) | struct | commands | — |
| [`CommandsReport`](commands.md#type-CommandsReport) | struct | commands | — |
| [`CommandStep`](commands.md#type-CommandStep) | struct | commands | — |
| [`CompassDesc`](ui.md#type-CompassDesc) | desc struct | ui | — |
| [`CompassPoi`](ui.md#type-CompassPoi) | desc struct | ui | — |
| [`ConeGeometryDesc`](render.md#type-ConeGeometryDesc) | desc struct | render | — |
| [`ConfirmationDesc`](ui.md#type-ConfirmationDesc) | desc struct | ui | — |
| [`ConfirmDesc`](ui.md#type-ConfirmDesc) | desc struct | ui | — |
| [`ConfirmResult`](ui.md#type-ConfirmResult) | struct | ui | — |
| [`ConflictPolicy`](input.md#type-ConflictPolicy) | enum | input | — |
| [`Contrast`](ui.md#type-Contrast) | enum | ui | — |
| [`ControlSize`](ui.md#type-ControlSize) | enum | ui | — |
| [`ControlSizes`](ui.md#type-ControlSizes) | struct | ui | — |
| [`CooldownDesc`](ui.md#type-CooldownDesc) | desc struct | ui | — |
| [`CoreAction`](commands.md#type-CoreAction) | enum | commands | — |
| [`CornerShape`](ui.md#type-CornerShape) | enum | ui | — |
| [`CountBadgeDesc`](ui.md#type-CountBadgeDesc) | desc struct | ui | — |
| [`CounterDesc`](ui.md#type-CounterDesc) | desc struct | ui | — |
| [`Cross`](ui.md#type-Cross) | enum | ui | — |
| [`CrosshairDesc`](ui.md#type-CrosshairDesc) | desc struct | ui | — |
| [`CrosshairStyle`](ui.md#type-CrosshairStyle) | enum | ui | — |
| [`Crumb`](ui.md#type-Crumb) | desc struct | ui | — |
| [`CsgGeometryDesc`](render.md#type-CsgGeometryDesc) | desc struct | render | — |
| [`CsgOp`](render.md#type-CsgOp) | enum | render | — |
| [`Currency`](ui.md#type-Currency) | desc struct | ui | — |
| [`CurveDesc`](path.md#type-CurveDesc) | desc struct | path | — |
| [`CutsceneDesc`](ui.md#type-CutsceneDesc) | desc struct | ui | — |
| [`CutsceneResult`](ui.md#type-CutsceneResult) | struct | ui | — |
| [`CyclerDesc`](ui.md#type-CyclerDesc) | desc struct | ui | — |
| [`CylinderGeometryDesc`](render.md#type-CylinderGeometryDesc) | desc struct | render | — |
| [`DamageDirectionDesc`](ui.md#type-DamageDirectionDesc) | desc struct | ui | — |
| [`DamageHit`](ui.md#type-DamageHit) | desc struct | ui | — |
| [`DamageHitDesc`](ui.md#type-DamageHitDesc) | desc struct | ui | — |
| [`DamageKind`](ui.md#type-DamageKind) | enum | ui | — |
| [`DamageNumbersDesc`](ui.md#type-DamageNumbersDesc) | desc struct | ui | — |
| [`DataGridDesc`](ui.md#type-DataGridDesc) | desc struct | ui | — |
| [`DataListDesc`](ui.md#type-DataListDesc) | desc struct | ui | — |
| [`DataListResult`](ui.md#type-DataListResult) | struct | ui | — |
| [`DataStateDesc`](ui.md#type-DataStateDesc) | desc struct | ui | — |
| [`DeadzoneDesc`](input.md#type-DeadzoneDesc) | desc struct | input | — |
| [`DebugDrawDesc`](debug.md#type-DebugDrawDesc) | desc struct | debug | — |
| [`DebugView`](render.md#type-DebugView) | enum | render | — |
| [`Declutter`](ui.md#type-Declutter) | enum | ui | — |
| [`defineComponent`](reflect.md#fn-defineComponent) | function | reflect | `og_reflect_define_component` |
| [`defineHook`](system.md#fn-defineHook) | function | system | `og_system_define_hook` |
| [`defineResource`](reflect.md#fn-defineResource) | function | reflect | `og_reflect_define_resource` |
| [`defineSystem`](system.md#fn-defineSystem) | function | system | `og_system_define_system` |
| [`DensityMode`](ui.md#type-DensityMode) | enum | ui | — |
| [`DescribedCamera`](probe.md#type-DescribedCamera) | struct | probe | — |
| [`DescribedEntity`](probe.md#type-DescribedEntity) | struct | probe | — |
| [`DescribedPrefab`](probe.md#type-DescribedPrefab) | struct | probe | — |
| [`DescribedUi`](probe.md#type-DescribedUi) | struct | probe | — |
| [`DescribedView`](probe.md#type-DescribedView) | struct | probe | — |
| [`Description`](probe.md#type-Description) | struct | probe | — |
| [`DesyncDump`](probe.md#type-DesyncDump) | struct | probe | — |
| [`DialogAction`](ui.md#type-DialogAction) | desc struct | ui | — |
| [`DialogDesc`](ui.md#type-DialogDesc) | desc struct | ui | — |
| [`DialogResult`](ui.md#type-DialogResult) | struct | ui | — |
| [`DialogTone`](ui.md#type-DialogTone) | enum | ui | — |
| [`DialogueDesc`](ui.md#type-DialogueDesc) | desc struct | ui | — |
| [`DialogueLine`](ui.md#type-DialogueLine) | desc struct | ui | — |
| [`DialogueResult`](ui.md#type-DialogueResult) | struct | ui | — |
| [`Direction`](ui.md#type-Direction) | enum | ui | — |
| [`DismissSpec`](ui.md#type-DismissSpec) | desc struct | ui | — |
| [`DisplayInfo`](render.md#type-DisplayInfo) | struct | render | — |
| [`DisplaySettings`](render.md#type-DisplaySettings) | desc struct | render | — |
| [`DitherPattern`](render.md#type-DitherPattern) | enum | render | — |
| [`DitherPattern()`](render.md#column-DitherPattern) | column kind | render | — |
| [`DofQuality`](render.md#type-DofQuality) | enum | render | — |
| [`DofQuality()`](render.md#column-DofQuality) | column kind | render | — |
| [`DpadDesc`](ui.md#type-DpadDesc) | desc struct | ui | — |
| [`DpadResult`](ui.md#type-DpadResult) | struct | ui | — |
| [`DragAxis`](ui.md#type-DragAxis) | enum | ui | — |
| [`DragSpec`](ui.md#type-DragSpec) | desc struct | ui | — |
| [`DrawDesc`](ui.md#type-DrawDesc) | desc struct | ui | — |
| [`DrawShape`](ui.md#type-DrawShape) | desc struct | ui | — |
| [`DRIVE_ALL`](physics.md#const-DRIVE_ALL) | constant | physics | — |
| [`DRIVE_FRONT`](physics.md#const-DRIVE_FRONT) | constant | physics | — |
| [`DRIVE_REAR`](physics.md#const-DRIVE_REAR) | constant | physics | — |
| [`DropdownDesc`](ui.md#type-DropdownDesc) | desc struct | ui | — |
| [`DropdownOption`](ui.md#type-DropdownOption) | desc struct | ui | — |
| [`DropdownResult`](ui.md#type-DropdownResult) | struct | ui | — |
| [`DropSpec`](ui.md#type-DropSpec) | desc struct | ui | — |
| [`DumpDesc`](probe.md#type-DumpDesc) | desc struct | probe | — |
| [`DumpKind`](probe.md#type-DumpKind) | enum | probe | — |
| [`DumpResult`](probe.md#type-DumpResult) | union | probe | — |
| [`Ease`](tween.md#type-Ease) | enum | tween | — |
| [`EaseKey`](ui.md#type-EaseKey) | struct | ui | — |
| [`EaseName`](ui.md#type-EaseName) | enum | ui | — |
| [`EdgeClampDesc`](ui.md#type-EdgeClampDesc) | desc struct | ui | — |
| [`Edges`](ui.md#type-Edges) | desc struct | ui | — |
| [`Effect`](ui.md#type-Effect) | union | ui | — |
| [`EffectLayerDesc`](ui.md#type-EffectLayerDesc) | desc struct | ui | — |
| [`ElementDesc`](ui.md#type-ElementDesc) | desc struct | ui | — |
| [`ElementProps`](ui.md#type-ElementProps) | desc struct | ui | — |
| [`Elevation`](ui.md#type-Elevation) | enum | ui | — |
| [`EmitFrom`](ui.md#type-EmitFrom) | enum | ui | — |
| [`EmitterDesc`](ui.md#type-EmitterDesc) | desc struct | ui | — |
| [`EmitterPreset`](ui.md#type-EmitterPreset) | enum | ui | — |
| [`EngineProbe`](probe.md#type-EngineProbe) | interface | probe | — |
| [`EngineProbe.capture`](probe.md#member-EngineProbe-capture) | method | probe | `og_engine_probe_capture` |
| [`EngineProbe.commands`](probe.md#member-EngineProbe-commands) | method | probe | `og_engine_probe_commands` |
| [`EngineProbe.describe`](probe.md#member-EngineProbe-describe) | method | probe | `og_engine_probe_describe` |
| [`EngineProbe.dump`](probe.md#member-EngineProbe-dump) | method | probe | `og_engine_probe_dump` |
| [`EngineProbe.expect`](probe.md#member-EngineProbe-expect) | method | probe | `og_engine_probe_expect` |
| [`EngineProbe.expectations`](probe.md#member-EngineProbe-expectations) | method | probe | `og_engine_probe_expectations` |
| [`EngineProbe.hello`](probe.md#member-EngineProbe-hello) | method | probe | `og_engine_probe_hello` |
| [`EngineProbe.listen`](probe.md#member-EngineProbe-listen) | method | probe | `og_engine_probe_listen` |
| [`EngineProbe.memory`](probe.md#member-EngineProbe-memory) | method | probe | `og_engine_probe_memory` |
| [`EngineProbe.reload`](probe.md#member-EngineProbe-reload) | method | probe | `og_engine_probe_reload` |
| [`EngineProbe.restore`](probe.md#member-EngineProbe-restore) | method | probe | `og_engine_probe_restore` |
| [`EngineProbe.saveScene`](probe.md#member-EngineProbe-saveScene) | method | probe | `og_engine_probe_save_scene` |
| [`EngineProbe.snapshot`](probe.md#member-EngineProbe-snapshot) | method | probe | `og_engine_probe_snapshot` |
| [`EngineProbe.sweep`](probe.md#member-EngineProbe-sweep) | method | probe | `og_engine_probe_sweep` |
| [`Entity`](world.md#type-Entity) | handle | world | — |
| [`Entity()`](world.md#column-Entity) | column kind | world | — |
| [`EntityDump`](probe.md#type-EntityDump) | struct | probe | — |
| [`EntityOperation`](probe.md#type-EntityOperation) | struct | probe | — |
| [`EntityPrefab`](probe.md#type-EntityPrefab) | struct | probe | — |
| [`EnvironmentMap`](render.md#type-EnvironmentMap) | handle | render | — |
| [`EnvironmentMap()`](render.md#column-EnvironmentMap) | column kind | render | — |
| [`ExpectOperation`](probe.md#type-ExpectOperation) | struct | probe | — |
| [`ExpectReport`](probe.md#type-ExpectReport) | struct | probe | — |
| [`ExpectResult`](probe.md#type-ExpectResult) | struct | probe | — |
| [`ExpectStatus`](probe.md#type-ExpectStatus) | enum | probe | — |
| [`ExtrudeGeometryDesc`](render.md#type-ExtrudeGeometryDesc) | desc struct | render | — |
| [`f32()`](reflect.md#column-f32) | column kind | reflect | — |
| [`f64()`](reflect.md#column-f64) | column kind | reflect | — |
| [`FactsFn`](game.md#type-FactsFn) | callback | game | — |
| [`FbmDesc`](procgen.md#type-FbmDesc) | desc struct | procgen | — |
| [`FieldDesc`](ui.md#type-FieldDesc) | desc struct | ui | — |
| [`FieldOptions`](reflect.md#type-FieldOptions) | desc struct | reflect | — |
| [`FieldResult`](ui.md#type-FieldResult) | struct | ui | — |
| [`Fill`](ui.md#type-Fill) | union | ui | — |
| [`FillDesc`](tilemap.md#type-FillDesc) | desc struct | tilemap | — |
| [`FixedSize`](ui.md#type-FixedSize) | struct | ui | — |
| [`FlashDesc`](tween.md#type-FlashDesc) | desc struct | tween | — |
| [`FlightHudDesc`](ui.md#type-FlightHudDesc) | desc struct | ui | — |
| [`FlipbookClipInfo`](anim.md#type-FlipbookClipInfo) | struct | anim | — |
| [`FlipbookInfo`](anim.md#type-FlipbookInfo) | struct | anim | — |
| [`FlipbookSheet`](anim.md#type-FlipbookSheet) | handle | anim | — |
| [`FlipbookSheet()`](anim.md#column-FlipbookSheet) | column kind | anim | — |
| [`FlipMode`](anim.md#type-FlipMode) | enum | anim | — |
| [`FloatAlign`](ui.md#type-FloatAlign) | enum | ui | — |
| [`FloatingTextDesc`](render.md#type-FloatingTextDesc) | desc struct | render | — |
| [`FloatSide`](ui.md#type-FloatSide) | enum | ui | — |
| [`FlowDesc`](nav.md#type-FlowDesc) | desc struct | nav | — |
| [`Font`](load.md#type-Font) | handle | load | — |
| [`Font()`](load.md#column-Font) | column kind | load | — |
| [`FontInfo`](load.md#type-FontInfo) | struct | load | — |
| [`FontRole`](ui.md#type-FontRole) | enum | ui | — |
| [`FrameCap`](render.md#type-FrameCap) | enum | render | — |
| [`FramesResult`](ui.md#type-FramesResult) | struct | ui | — |
| [`Frustum`](math.md#type-Frustum) | struct | math | — |
| [`frustum`](math.md#value-frustum) | value | math | — |
| [`FrustumMath`](math.md#type-FrustumMath) | interface | math | — |
| [`FrustumMath.containsPoint`](math.md#member-FrustumMath-containsPoint) | method | math | — |
| [`FrustumMath.fromCamera`](math.md#member-FrustumMath-fromCamera) | method | math | — |
| [`FrustumMath.fromProjectionMatrix`](math.md#member-FrustumMath-fromProjectionMatrix) | method | math | — |
| [`FrustumMath.intersectsBox`](math.md#member-FrustumMath-intersectsBox) | method | math | — |
| [`FrustumMath.intersectsSphere`](math.md#member-FrustumMath-intersectsSphere) | method | math | — |
| [`Game`](game.md#type-Game) | interface | game | — |
| [`Game.systems`](game.md#member-Game-systems) | property | game | `og_game_get_systems` |
| [`Game.tick`](game.md#member-Game-tick) | property | game | `og_game_get_tick` |
| [`GameDesc`](game.md#type-GameDesc) | desc struct | game | — |
| [`Gamepad.axis`](input.md#member-Gamepad-axis) | method | input | `og_gamepad_axis` |
| [`Gamepad.connected`](input.md#member-Gamepad-connected) | method | input | `og_gamepad_connected` |
| [`Gamepad.down`](input.md#member-Gamepad-down) | method | input | `og_gamepad_down` |
| [`Gamepad.pressed`](input.md#member-Gamepad-pressed) | method | input | `og_gamepad_pressed` |
| [`Gamepad.released`](input.md#member-Gamepad-released) | method | input | `og_gamepad_released` |
| [`GamepadAxis`](input.md#type-GamepadAxis) | enum | input | — |
| [`GamepadButton`](input.md#type-GamepadButton) | enum | input | — |
| [`GamepadInput`](input.md#type-GamepadInput) | struct | input | — |
| [`GameSettings`](ui.md#type-GameSettings) | desc struct | ui | — |
| [`Generator`](ui.md#type-Generator) | desc struct | ui | — |
| [`Geometry`](render.md#type-Geometry) | handle | render | — |
| [`Geometry()`](render.md#column-Geometry) | column kind | render | — |
| [`GeometryHit`](render.md#type-GeometryHit) | struct | render | — |
| [`GeometryInfo`](render.md#type-GeometryInfo) | struct | render | — |
| [`GeometryPart`](render.md#type-GeometryPart) | desc struct | render | — |
| [`GestureInfo`](ui.md#type-GestureInfo) | struct | ui | — |
| [`GlassWeight`](ui.md#type-GlassWeight) | enum | ui | — |
| [`GlowEffect`](ui.md#type-GlowEffect) | desc struct | ui | — |
| [`GltfDesc`](load.md#type-GltfDesc) | desc struct | load | — |
| [`GltfInfo`](load.md#type-GltfInfo) | struct | load | — |
| [`GpuMemory`](render.md#type-GpuMemory) | struct | render | — |
| [`Gradient`](ui.md#type-Gradient) | desc struct | ui | — |
| [`GradientSpread`](ui.md#type-GradientSpread) | enum | ui | — |
| [`GradientStop`](ui.md#type-GradientStop) | desc struct | ui | — |
| [`GradientType`](ui.md#type-GradientType) | enum | ui | — |
| [`GraphAsset`](anim.md#type-GraphAsset) | handle | anim | — |
| [`GraphAsset()`](anim.md#column-GraphAsset) | column kind | anim | — |
| [`GraphDesc`](anim.md#type-GraphDesc) | desc struct | anim | — |
| [`GraphicsQuality`](render.md#type-GraphicsQuality) | enum | render | — |
| [`GraphLink`](ui.md#type-GraphLink) | desc struct | ui | — |
| [`GraphNode`](ui.md#type-GraphNode) | desc struct | ui | — |
| [`GraphNodeShape`](ui.md#type-GraphNodeShape) | enum | ui | — |
| [`GraphNodeState`](ui.md#type-GraphNodeState) | enum | ui | — |
| [`GridDesc`](ui.md#type-GridDesc) | desc struct | ui | — |
| [`GridFlow`](ui.md#type-GridFlow) | enum | ui | — |
| [`GridPlace`](ui.md#type-GridPlace) | desc struct | ui | — |
| [`GridSpec`](ui.md#type-GridSpec) | desc struct | ui | — |
| [`GroundDesc`](ui.md#type-GroundDesc) | desc struct | ui | — |
| [`HapticEvent`](ui.md#type-HapticEvent) | struct | ui | — |
| [`Heard`](probe.md#type-Heard) | struct | probe | — |
| [`Hello`](probe.md#type-Hello) | struct | probe | — |
| [`HitKind`](ui.md#type-HitKind) | enum | ui | — |
| [`HoldButtonDesc`](ui.md#type-HoldButtonDesc) | desc struct | ui | — |
| [`HoldKey`](ui.md#type-HoldKey) | struct | ui | — |
| [`HoldResult`](ui.md#type-HoldResult) | struct | ui | — |
| [`Hook`](system.md#type-Hook) | interface | system | — |
| [`Hook.name`](system.md#member-Hook-name) | property | system | `og_hook_get_name` |
| [`HookDesc`](system.md#type-HookDesc) | desc struct | system | — |
| [`HookEvent`](system.md#type-HookEvent) | enum | system | — |
| [`HookRun`](system.md#type-HookRun) | callback | system | — |
| [`HudResult`](ui.md#type-HudResult) | struct | ui | — |
| [`HudTimerDesc`](ui.md#type-HudTimerDesc) | desc struct | ui | — |
| [`HumanBone`](anim.md#type-HumanBone) | struct | anim | — |
| [`Humanoid`](anim.md#type-Humanoid) | struct | anim | — |
| [`HumanoidDesc`](anim.md#type-HumanoidDesc) | desc struct | anim | — |
| [`i32()`](reflect.md#column-i32) | column kind | reflect | — |
| [`IconAtlasDesc`](ui.md#type-IconAtlasDesc) | desc struct | ui | — |
| [`IconButtonDesc`](ui.md#type-IconButtonDesc) | desc struct | ui | — |
| [`IconDesc`](ui.md#type-IconDesc) | desc struct | ui | — |
| [`IconMarkDesc`](ui.md#type-IconMarkDesc) | desc struct | ui | — |
| [`IconPlate`](ui.md#type-IconPlate) | enum | ui | — |
| [`IdleHudDesc`](ui.md#type-IdleHudDesc) | desc struct | ui | — |
| [`IkSolver`](anim.md#type-IkSolver) | enum | anim | — |
| [`IkSolver()`](anim.md#column-IkSolver) | column kind | anim | — |
| [`IkSpace`](anim.md#type-IkSpace) | enum | anim | — |
| [`IkSpace()`](anim.md#column-IkSpace) | column kind | anim | — |
| [`ImageDesc`](ui.md#type-ImageDesc) | desc struct | ui | — |
| [`ImageFill`](ui.md#type-ImageFill) | desc struct | ui | — |
| [`ImageFit`](ui.md#type-ImageFit) | enum | ui | — |
| [`InjectedInput`](input.md#type-InjectedInput) | union | input | — |
| [`Input.axis`](input.md#member-Input-axis) | method | input | `og_input_axis` |
| [`Input.axis2`](input.md#member-Input-axis2) | method | input | `og_input_axis2` |
| [`Input.buttonDown`](input.md#member-Input-buttonDown) | method | input | `og_input_button_down` |
| [`Input.buttonPressed`](input.md#member-Input-buttonPressed) | method | input | `og_input_button_pressed` |
| [`Input.buttonReleased`](input.md#member-Input-buttonReleased) | method | input | `og_input_button_released` |
| [`Input.down`](input.md#member-Input-down) | method | input | `og_input_down` |
| [`Input.pressed`](input.md#member-Input-pressed) | method | input | `og_input_pressed` |
| [`Input.released`](input.md#member-Input-released) | method | input | `og_input_released` |
| [`InputAreaDesc`](ui.md#type-InputAreaDesc) | desc struct | ui | — |
| [`InputBinding`](input.md#type-InputBinding) | desc struct | input | — |
| [`InputDesc`](net.md#type-InputDesc) | desc struct | net | — |
| [`InputDevice`](input.md#type-InputDevice) | enum | input | — |
| [`InputFieldDesc`](ui.md#type-InputFieldDesc) | desc struct | ui | — |
| [`InputGlyph`](input.md#type-InputGlyph) | struct | input | — |
| [`InputOperation`](probe.md#type-InputOperation) | struct | probe | — |
| [`InputResult`](ui.md#type-InputResult) | struct | ui | — |
| [`Insets`](ui.md#type-Insets) | struct | ui | — |
| [`InstantSpec`](ui.md#type-InstantSpec) | struct | ui | — |
| [`InteractionPromptDesc`](ui.md#type-InteractionPromptDesc) | desc struct | ui | — |
| [`InventoryDesc`](ui.md#type-InventoryDesc) | desc struct | ui | — |
| [`InventoryItem`](ui.md#type-InventoryItem) | desc struct | ui | — |
| [`InventoryResult`](ui.md#type-InventoryResult) | struct | ui | — |
| [`JOINT_BALL`](physics.md#const-JOINT_BALL) | constant | physics | — |
| [`JOINT_FIXED`](physics.md#const-JOINT_FIXED) | constant | physics | — |
| [`JOINT_PRISMATIC`](physics.md#const-JOINT_PRISMATIC) | constant | physics | — |
| [`JOINT_REVOLUTE`](physics.md#const-JOINT_REVOLUTE) | constant | physics | — |
| [`JOINT_ROPE`](physics.md#const-JOINT_ROPE) | constant | physics | — |
| [`JOINT_SPRING`](physics.md#const-JOINT_SPRING) | constant | physics | — |
| [`JoystickDesc`](ui.md#type-JoystickDesc) | desc struct | ui | — |
| [`JoystickResult`](ui.md#type-JoystickResult) | struct | ui | — |
| [`JudgementDesc`](ui.md#type-JudgementDesc) | desc struct | ui | — |
| [`JudgementEvent`](ui.md#type-JudgementEvent) | desc struct | ui | — |
| [`KeyAction`](input.md#type-KeyAction) | enum | input | — |
| [`KeyBinding`](ui.md#type-KeyBinding) | desc struct | ui | — |
| [`KeyBindingResult`](ui.md#type-KeyBindingResult) | struct | ui | — |
| [`KeyBindingRowDesc`](ui.md#type-KeyBindingRowDesc) | desc struct | ui | — |
| [`KeyboardDesc`](ui.md#type-KeyboardDesc) | desc struct | ui | — |
| [`KeyboardLayout`](ui.md#type-KeyboardLayout) | enum | ui | — |
| [`KeyboardResult`](ui.md#type-KeyboardResult) | struct | ui | — |
| [`KeyEase`](ui.md#type-KeyEase) | union | ui | — |
| [`KeyInput`](input.md#type-KeyInput) | struct | input | — |
| [`KeyLabel`](input.md#type-KeyLabel) | desc struct | input | — |
| [`KeyValueRow`](ui.md#type-KeyValueRow) | desc struct | ui | — |
| [`KeyValueRowsDesc`](ui.md#type-KeyValueRowsDesc) | desc struct | ui | — |
| [`KillfeedDesc`](ui.md#type-KillfeedDesc) | desc struct | ui | — |
| [`KillfeedEntry`](ui.md#type-KillfeedEntry) | desc struct | ui | — |
| [`LabelDesc`](ui.md#type-LabelDesc) | desc struct | ui | — |
| [`LabelSide`](ui.md#type-LabelSide) | enum | ui | — |
| [`LanguageOption`](ui.md#type-LanguageOption) | desc struct | ui | — |
| [`LatheGeometryDesc`](render.md#type-LatheGeometryDesc) | desc struct | render | — |
| [`LayerDesc`](ui.md#type-LayerDesc) | desc struct | ui | — |
| [`LayoutMode`](ui.md#type-LayoutMode) | enum | ui | — |
| [`LeaderboardDesc`](ui.md#type-LeaderboardDesc) | desc struct | ui | — |
| [`LeaderboardEntry`](ui.md#type-LeaderboardEntry) | desc struct | ui | — |
| [`LeaderboardQueryDesc`](platform.md#type-LeaderboardQueryDesc) | desc struct | platform | — |
| [`LeaderboardResult`](ui.md#type-LeaderboardResult) | struct | ui | — |
| [`LeaderboardSubmitDesc`](platform.md#type-LeaderboardSubmitDesc) | desc struct | platform | — |
| [`LevelInfo`](ui.md#type-LevelInfo) | desc struct | ui | — |
| [`LevelSelectDesc`](ui.md#type-LevelSelectDesc) | desc struct | ui | — |
| [`LevelSelectResult`](ui.md#type-LevelSelectResult) | struct | ui | — |
| [`LinearKey`](ui.md#type-LinearKey) | struct | ui | — |
| [`LineCap`](ui.md#type-LineCap) | enum | ui | — |
| [`LineJoin`](ui.md#type-LineJoin) | enum | ui | — |
| [`LineMode`](render.md#type-LineMode) | enum | render | — |
| [`LineMode()`](render.md#column-LineMode) | column kind | render | — |
| [`LinesDesc`](render.md#type-LinesDesc) | desc struct | render | — |
| [`list()`](reflect.md#column-list) | column kind | reflect | — |
| [`ListDesc`](ui.md#type-ListDesc) | desc struct | ui | — |
| [`ListRange`](ui.md#type-ListRange) | struct | ui | — |
| [`ListResult`](ui.md#type-ListResult) | struct | ui | — |
| [`LoadingDesc`](ui.md#type-LoadingDesc) | desc struct | ui | — |
| [`LoadSlotDesc`](platform.md#type-LoadSlotDesc) | desc struct | platform | — |
| [`LoadStatus`](load.md#type-LoadStatus) | enum | load | — |
| [`LocaleInfo`](i18n.md#type-LocaleInfo) | struct | i18n | — |
| [`LOCK_ROTATION_X`](physics.md#const-LOCK_ROTATION_X) | constant | physics | — |
| [`LOCK_ROTATION_Y`](physics.md#const-LOCK_ROTATION_Y) | constant | physics | — |
| [`LOCK_ROTATION_Z`](physics.md#const-LOCK_ROTATION_Z) | constant | physics | — |
| [`LOCK_TRANSLATION_X`](physics.md#const-LOCK_TRANSLATION_X) | constant | physics | — |
| [`LOCK_TRANSLATION_Y`](physics.md#const-LOCK_TRANSLATION_Y) | constant | physics | — |
| [`LOCK_TRANSLATION_Z`](physics.md#const-LOCK_TRANSLATION_Z) | constant | physics | — |
| [`LockstepSeat`](net.md#type-LockstepSeat) | struct | net | — |
| [`LockstepStatus`](net.md#type-LockstepStatus) | struct | net | — |
| [`Look`](render.md#type-Look) | handle | render | — |
| [`Look()`](render.md#column-Look) | column kind | render | — |
| [`LookInfo`](render.md#type-LookInfo) | struct | render | — |
| [`LootLabelDesc`](ui.md#type-LootLabelDesc) | desc struct | ui | — |
| [`LootLabelResult`](ui.md#type-LootLabelResult) | struct | ui | — |
| [`LowHealthDesc`](ui.md#type-LowHealthDesc) | desc struct | ui | — |
| [`Lut`](render.md#type-Lut) | handle | render | — |
| [`Lut()`](render.md#column-Lut) | column kind | render | — |
| [`MainMenuDesc`](ui.md#type-MainMenuDesc) | desc struct | ui | — |
| [`MapMarker`](ui.md#type-MapMarker) | desc struct | ui | — |
| [`MapMarkerItem`](ui.md#type-MapMarkerItem) | desc struct | ui | — |
| [`MapShape`](ui.md#type-MapShape) | enum | ui | — |
| [`MarkerKind`](ui.md#type-MarkerKind) | enum | ui | — |
| [`MaskDesc`](ui.md#type-MaskDesc) | desc struct | ui | — |
| [`MassEase`](ui.md#type-MassEase) | enum | ui | — |
| [`MassInstance`](ui.md#type-MassInstance) | desc struct | ui | — |
| [`MassKey`](ui.md#type-MassKey) | desc struct | ui | — |
| [`MassKind`](ui.md#type-MassKind) | enum | ui | — |
| [`MassSpace`](ui.md#type-MassSpace) | enum | ui | — |
| [`MassTemplateDesc`](ui.md#type-MassTemplateDesc) | desc struct | ui | — |
| [`mat4`](math.md#value-mat4) | value | math | — |
| [`Material`](render.md#type-Material) | handle | render | — |
| [`Material()`](render.md#column-Material) | column kind | render | — |
| [`MaterialName`](ui.md#type-MaterialName) | enum | ui | — |
| [`MaterialSurface`](render.md#type-MaterialSurface) | enum | render | — |
| [`MaterialSurface()`](render.md#column-MaterialSurface) | column kind | render | — |
| [`Matrix4`](math.md#type-Matrix4) | struct | math | — |
| [`Matrix4()`](reflect.md#column-Matrix4) | column kind | reflect | — |
| [`Matrix4Math`](math.md#type-Matrix4Math) | interface | math | — |
| [`Matrix4Math.compose`](math.md#member-Matrix4Math-compose) | method | math | — |
| [`Matrix4Math.invert`](math.md#member-Matrix4Math-invert) | method | math | — |
| [`Matrix4Math.multiply`](math.md#member-Matrix4Math-multiply) | method | math | — |
| [`Matrix4Math.transformDirection`](math.md#member-Matrix4Math-transformDirection) | method | math | — |
| [`Matrix4Math.transformPoint`](math.md#member-Matrix4Math-transformPoint) | method | math | — |
| [`MAX_CLUSTER_LIGHTS`](render.md#const-MAX_CLUSTER_LIGHTS) | constant | render | — |
| [`MAX_GAMEPADS`](input.md#const-MAX_GAMEPADS) | constant | input | — |
| [`MAX_LIGHTS`](render.md#const-MAX_LIGHTS) | constant | render | — |
| [`MAX_POINTERS`](input.md#const-MAX_POINTERS) | constant | input | — |
| [`MAX_RAYCAST_HITS`](physics.md#const-MAX_RAYCAST_HITS) | constant | physics | — |
| [`MAX_REFLECTION_PROBES`](render.md#const-MAX_REFLECTION_PROBES) | constant | render | — |
| [`MenuChoice`](ui.md#type-MenuChoice) | struct | ui | — |
| [`MenuDesc`](ui.md#type-MenuDesc) | desc struct | ui | — |
| [`MenuEntry`](ui.md#type-MenuEntry) | desc struct | ui | — |
| [`MenuEntryKind`](ui.md#type-MenuEntryKind) | enum | ui | — |
| [`MenuItem`](ui.md#type-MenuItem) | desc struct | ui | — |
| [`MenuResult`](ui.md#type-MenuResult) | struct | ui | — |
| [`MergeGeometriesDesc`](render.md#type-MergeGeometriesDesc) | desc struct | render | — |
| [`MeshDesc`](physics.md#type-MeshDesc) | desc struct | physics | — |
| [`MetaDesc`](ui.md#type-MetaDesc) | desc struct | ui | — |
| [`MetaKind`](ui.md#type-MetaKind) | enum | ui | — |
| [`MeterDesc`](ui.md#type-MeterDesc) | desc struct | ui | — |
| [`MeterKind`](ui.md#type-MeterKind) | enum | ui | — |
| [`MigrateFn`](world.md#type-MigrateFn) | callback | world | — |
| [`MinimapDesc`](ui.md#type-MinimapDesc) | desc struct | ui | — |
| [`MinimapResult`](ui.md#type-MinimapResult) | struct | ui | — |
| [`MinimapViewDesc`](ui.md#type-MinimapViewDesc) | desc struct | ui | — |
| [`MirrorAxis`](render.md#type-MirrorAxis) | enum | render | — |
| [`MirrorDesc`](anim.md#type-MirrorDesc) | desc struct | anim | — |
| [`ModalDesc`](ui.md#type-ModalDesc) | desc struct | ui | — |
| [`ModalResult`](ui.md#type-ModalResult) | struct | ui | — |
| [`Modifiers`](input.md#type-Modifiers) | struct | input | — |
| [`MotionToken`](ui.md#type-MotionToken) | enum | ui | — |
| [`MOTOR_NONE`](physics.md#const-MOTOR_NONE) | constant | physics | — |
| [`MOTOR_POSITION`](physics.md#const-MOTOR_POSITION) | constant | physics | — |
| [`MOTOR_VELOCITY`](physics.md#const-MOTOR_VELOCITY) | constant | physics | — |
| [`MouseDesc`](input.md#type-MouseDesc) | desc struct | input | — |
| [`MouseInfo`](input.md#type-MouseInfo) | struct | input | — |
| [`NameplateDesc`](ui.md#type-NameplateDesc) | desc struct | ui | — |
| [`NAV_ARRIVED`](nav.md#const-NAV_ARRIVED) | constant | nav | — |
| [`NAV_IDLE`](nav.md#const-NAV_IDLE) | constant | nav | — |
| [`NAV_MOVING`](nav.md#const-NAV_MOVING) | constant | nav | — |
| [`NAV_ON_LINK`](nav.md#const-NAV_ON_LINK) | constant | nav | — |
| [`NAV_PARTIAL`](nav.md#const-NAV_PARTIAL) | constant | nav | — |
| [`NAV_UNREACHABLE`](nav.md#const-NAV_UNREACHABLE) | constant | nav | — |
| [`NavAlgorithm`](nav.md#type-NavAlgorithm) | enum | nav | — |
| [`NavAreaDesc`](nav.md#type-NavAreaDesc) | desc struct | nav | — |
| [`NavBox`](nav.md#type-NavBox) | desc struct | nav | — |
| [`NavCell`](nav.md#type-NavCell) | struct | nav | — |
| [`NavGridDesc`](nav.md#type-NavGridDesc) | desc struct | nav | — |
| [`Navigation`](nav.md#type-Navigation) | handle | nav | — |
| [`Navigation()`](nav.md#column-Navigation) | column kind | nav | — |
| [`NavIntent`](ui.md#type-NavIntent) | enum | ui | — |
| [`NavLinkDesc`](nav.md#type-NavLinkDesc) | desc struct | nav | — |
| [`NavLinkKind`](nav.md#type-NavLinkKind) | enum | nav | — |
| [`NavMeshDesc`](nav.md#type-NavMeshDesc) | desc struct | nav | — |
| [`NavMeshInfo`](nav.md#type-NavMeshInfo) | struct | nav | — |
| [`NavPath`](nav.md#type-NavPath) | struct | nav | — |
| [`NavRebuild`](nav.md#type-NavRebuild) | struct | nav | — |
| [`NavShape`](nav.md#type-NavShape) | desc struct | nav | — |
| [`NavShapeKind`](nav.md#type-NavShapeKind) | enum | nav | — |
| [`Neighbour`](world.md#type-Neighbour) | struct | world | — |
| [`NetMode`](net.md#type-NetMode) | enum | net | — |
| [`NetMode()`](net.md#column-NetMode) | column kind | net | — |
| [`NodeGraphDesc`](ui.md#type-NodeGraphDesc) | desc struct | ui | — |
| [`NodeGraphResult`](ui.md#type-NodeGraphResult) | struct | ui | — |
| [`NoiseDesc`](ai.md#type-NoiseDesc) | desc struct | ai | — |
| [`NoiseGridDesc`](procgen.md#type-NoiseGridDesc) | desc struct | procgen | — |
| [`NoiseKind`](procgen.md#type-NoiseKind) | enum | procgen | — |
| [`Notation`](ui.md#type-Notation) | enum | ui | — |
| [`NotificationItem`](ui.md#type-NotificationItem) | desc struct | ui | — |
| [`NotificationsDesc`](ui.md#type-NotificationsDesc) | desc struct | ui | — |
| [`NotificationsResult`](ui.md#type-NotificationsResult) | struct | ui | — |
| [`NotificationVariant`](ui.md#type-NotificationVariant) | enum | ui | — |
| [`NumberCounterDesc`](ui.md#type-NumberCounterDesc) | desc struct | ui | — |
| [`NumberFormat`](ui.md#type-NumberFormat) | enum | ui | — |
| [`NumberFormatDesc`](ui.md#type-NumberFormatDesc) | desc struct | ui | — |
| [`NumberResult`](ui.md#type-NumberResult) | struct | ui | — |
| [`NumberStepperDesc`](ui.md#type-NumberStepperDesc) | desc struct | ui | — |
| [`ObjectiveQuest`](ui.md#type-ObjectiveQuest) | desc struct | ui | — |
| [`ObjectiveStep`](ui.md#type-ObjectiveStep) | desc struct | ui | — |
| [`ObjectiveTrackerDesc`](ui.md#type-ObjectiveTrackerDesc) | desc struct | ui | — |
| [`og.A11yAction`](input.md#record-A11yAction) | event | input | — |
| [`og.Afterimage`](render.md#record-Afterimage) | component | render | — |
| [`og.ai.describe`](ai.md#fn-describe) | function | ai | `og_ai_describe` |
| [`og.ai.finish`](ai.md#fn-finish) | function | ai | `og_ai_finish` |
| [`og.ai.get`](ai.md#fn-get) | function | ai | `og_ai_get` |
| [`og.ai.load`](ai.md#fn-load) | function | ai | `og_ai_load` |
| [`og.ai.noise`](ai.md#fn-noise) | function | ai | `og_ai_noise` |
| [`og.ai.parse`](ai.md#fn-parse) | function | ai | `og_ai_parse` |
| [`og.ai.restart`](ai.md#fn-restart) | function | ai | `og_ai_restart` |
| [`og.ai.set`](ai.md#fn-set) | function | ai | `og_ai_set` |
| [`og.AiActionStarted`](ai.md#record-AiActionStarted) | event | ai | — |
| [`og.AiActionStopped`](ai.md#record-AiActionStopped) | event | ai | — |
| [`og.AiAgent`](ai.md#record-AiAgent) | component | ai | — |
| [`og.AiChanged`](ai.md#record-AiChanged) | event | ai | — |
| [`og.anim.clip`](anim.md#fn-clip) | function | anim | — |
| [`og.anim.clip`](anim.md#fn-clip) | function | anim | — |
| [`og.anim.clip`](anim.md#fn-clip) | function | anim | — |
| [`og.anim.graph`](anim.md#fn-graph) | function | anim | `og_anim_graph` |
| [`og.anim.humanoid`](anim.md#fn-humanoid) | function | anim | `og_anim_humanoid` |
| [`og.anim.mirror`](anim.md#fn-mirror) | function | anim | `og_anim_mirror` |
| [`og.anim.retarget`](anim.md#fn-retarget) | function | anim | `og_anim_retarget` |
| [`og.anim.rig2dInfo`](anim.md#fn-rig2dInfo) | function | anim | `og_anim_rig2d_info` |
| [`og.anim.skeleton2dBoxes`](anim.md#fn-skeleton2dBoxes) | function | anim | `og_anim_skeleton2d_boxes` |
| [`og.AnimationEvent`](anim.md#record-AnimationEvent) | event | anim | — |
| [`og.AnimationGraph`](anim.md#record-AnimationGraph) | component | anim | — |
| [`og.AnimationPlayer`](anim.md#record-AnimationPlayer) | component | anim | — |
| [`og.ArtCamera`](art.md#record-ArtCamera) | resource | art | — |
| [`og.ArtLine`](art.md#record-ArtLine) | resource | art | — |
| [`og.ArtMotion`](art.md#record-ArtMotion) | resource | art | — |
| [`og.ArtPalette`](art.md#record-ArtPalette) | resource | art | — |
| [`og.ArtPixels`](art.md#record-ArtPixels) | resource | art | — |
| [`og.ArtShape`](art.md#record-ArtShape) | resource | art | — |
| [`og.ArtTexels`](art.md#record-ArtTexels) | resource | art | — |
| [`og.AssetFailed`](load.md#record-AssetFailed) | event | load | — |
| [`og.AssetReady`](load.md#record-AssetReady) | event | load | — |
| [`og.atmosphere.timeOfDay`](atmosphere.md#fn-timeOfDay) | function | atmosphere | `og_atmosphere_time_of_day` |
| [`og.Audio`](audio.md#record-Audio) | resource | audio | — |
| [`og.audio.createSound`](audio.md#fn-createSound) | function | audio | `og_audio_create_sound` |
| [`og.audio.outputLatency`](audio.md#fn-outputLatency) | function | audio | `og_audio_output_latency` |
| [`og.AudioBusFx`](audio.md#record-AudioBusFx) | component | audio | — |
| [`og.AudioBusNode`](audio.md#record-AudioBusNode) | component | audio | — |
| [`og.AudioEvent`](audio.md#record-AudioEvent) | component | audio | — |
| [`og.AudioListener`](audio.md#record-AudioListener) | component | audio | — |
| [`og.AudioSnapshot`](audio.md#record-AudioSnapshot) | component | audio | — |
| [`og.AudioSource`](audio.md#record-AudioSource) | component | audio | — |
| [`og.Beam`](render.md#record-Beam) | component | render | — |
| [`og.Blendshapes`](anim.md#record-Blendshapes) | component | anim | — |
| [`og.BodySlept`](physics.md#record-BodySlept) | event | physics | — |
| [`og.BodyWoke`](physics.md#record-BodyWoke) | event | physics | — |
| [`og.Camera`](camera.md#record-Camera) | resource | camera | — |
| [`og.camera.kick`](camera.md#fn-kick) | function | camera | `og_camera_kick` |
| [`og.camera.punch`](camera.md#fn-punch) | function | camera | `og_camera_punch` |
| [`og.camera.shake`](camera.md#fn-shake) | function | camera | `og_camera_shake` |
| [`og.CameraRail`](camera.md#record-CameraRail) | component | camera | — |
| [`og.CameraView`](render.md#record-CameraView) | component | render | — |
| [`og.CameraZone`](camera.md#record-CameraZone) | component | camera | — |
| [`og.CharacterController`](physics.md#record-CharacterController) | component | physics | — |
| [`og.CharacterEvent`](physics.md#record-CharacterEvent) | event | physics | — |
| [`og.Cloth`](physics.md#record-Cloth) | component | physics | — |
| [`og.Collider`](physics.md#record-Collider) | component | physics | — |
| [`og.CollisionEnded`](physics.md#record-CollisionEnded) | event | physics | — |
| [`og.CollisionStarted`](physics.md#record-CollisionStarted) | event | physics | — |
| [`og.commands.help`](commands.md#fn-help) | function | commands | `og_commands_help` |
| [`og.commands.parse`](commands.md#fn-parse) | function | commands | `og_commands_parse` |
| [`og.commands.register`](commands.md#fn-register) | function | commands | `og_commands_register` |
| [`og.commands.schedule`](commands.md#fn-schedule) | function | commands | `og_commands_schedule` |
| [`og.Contact`](physics.md#record-Contact) | event | physics | — |
| [`og.ContactHook`](physics.md#record-ContactHook) | component | physics | — |
| [`og.Crowd`](nav.md#record-Crowd) | component | nav | — |
| [`og.DayNight`](atmosphere.md#record-DayNight) | resource | atmosphere | — |
| [`og.debug.arrow`](debug.md#fn-arrow) | function | debug | `og_debug_arrow` |
| [`og.debug.axes`](debug.md#fn-axes) | function | debug | `og_debug_axes` |
| [`og.debug.box`](debug.md#fn-box) | function | debug | `og_debug_box` |
| [`og.debug.clear`](debug.md#fn-clear) | function | debug | `og_debug_clear` |
| [`og.debug.enabled`](debug.md#fn-enabled) | function | debug | `og_debug_enabled` |
| [`og.debug.line`](debug.md#fn-line) | function | debug | `og_debug_line` |
| [`og.debug.setEnabled`](debug.md#fn-setEnabled) | function | debug | `og_debug_set_enabled` |
| [`og.debug.setView`](debug.md#fn-setView) | function | debug | `og_debug_set_view` |
| [`og.debug.sphere`](debug.md#fn-sphere) | function | debug | `og_debug_sphere` |
| [`og.debug.text`](debug.md#fn-text) | function | debug | `og_debug_text` |
| [`og.debug.view`](debug.md#fn-view) | function | debug | `og_debug_view` |
| [`og.Decal`](render.md#record-Decal) | component | render | — |
| [`og.Desync`](net.md#record-Desync) | event | net | — |
| [`og.DirectionalLight`](render.md#record-DirectionalLight) | component | render | — |
| [`og.Environment`](render.md#record-Environment) | resource | render | — |
| [`og.Flipbook`](anim.md#record-Flipbook) | component | anim | — |
| [`og.FlipbookEvent`](anim.md#record-FlipbookEvent) | event | anim | — |
| [`og.Fluid2D`](physics.md#record-Fluid2D) | component | physics | — |
| [`og.Fluid3D`](physics.md#record-Fluid3D) | component | physics | — |
| [`og.Fog`](atmosphere.md#record-Fog) | resource | atmosphere | — |
| [`og.Follow2D`](camera.md#record-Follow2D) | component | camera | — |
| [`og.FollowCamera`](camera.md#record-FollowCamera) | component | camera | — |
| [`og.FootLock`](anim.md#record-FootLock) | component | anim | — |
| [`og.game`](game.md#fn-game) | function | game | `og_game_game` |
| [`og.Gamepad`](input.md#record-Gamepad) | resource | input | — |
| [`og.i18n.has`](i18n.md#fn-has) | function | i18n | `og_i18n_has` |
| [`og.i18n.load`](i18n.md#fn-load) | function | i18n | `og_i18n_load` |
| [`og.i18n.locale`](i18n.md#fn-locale) | function | i18n | `og_i18n_locale` |
| [`og.i18n.setLocale`](i18n.md#fn-setLocale) | function | i18n | `og_i18n_set_locale` |
| [`og.i18n.t`](i18n.md#fn-t) | function | i18n | `og_i18n_t` |
| [`og.IkChain`](anim.md#record-IkChain) | component | anim | — |
| [`og.IkTarget2D`](anim.md#record-IkTarget2D) | component | anim | — |
| [`og.Input`](input.md#record-Input) | resource | input | — |
| [`og.input.action`](input.md#fn-action) | function | input | `og_input_action` |
| [`og.input.actionDown`](input.md#fn-actionDown) | function | input | `og_input_action_down` |
| [`og.input.actionHeld`](input.md#fn-actionHeld) | function | input | `og_input_action_held` |
| [`og.input.actionPressed`](input.md#fn-actionPressed) | function | input | `og_input_action_pressed` |
| [`og.input.actionReleased`](input.md#fn-actionReleased) | function | input | `og_input_action_released` |
| [`og.input.actions`](input.md#fn-actions) | function | input | `og_input_actions` |
| [`og.input.actionValue`](input.md#fn-actionValue) | function | input | `og_input_action_value` |
| [`og.input.actionVector`](input.md#fn-actionVector) | function | input | `og_input_action_vector` |
| [`og.input.applyCapture`](input.md#fn-applyCapture) | function | input | `og_input_apply_capture` |
| [`og.input.cancelCapture`](input.md#fn-cancelCapture) | function | input | `og_input_cancel_capture` |
| [`og.input.capture`](input.md#fn-capture) | function | input | `og_input_capture` |
| [`og.input.conflicts`](input.md#fn-conflicts) | function | input | `og_input_conflicts` |
| [`og.input.contexts`](input.md#fn-contexts) | function | input | `og_input_contexts` |
| [`og.input.controlGlyph`](input.md#fn-controlGlyph) | function | input | `og_input_control_glyph` |
| [`og.input.defineAction`](input.md#fn-defineAction) | function | input | `og_input_define_action` |
| [`og.input.device`](input.md#fn-device) | function | input | `og_input_device` |
| [`og.input.glyph`](input.md#fn-glyph) | function | input | `og_input_glyph` |
| [`og.input.glyphs`](input.md#fn-glyphs) | function | input | `og_input_glyphs` |
| [`og.input.keyboardLayout`](input.md#fn-keyboardLayout) | function | input | `og_input_keyboard_layout` |
| [`og.input.loadBindings`](input.md#fn-loadBindings) | function | input | `og_input_load_bindings` |
| [`og.input.mouse`](input.md#fn-mouse) | function | input | `og_input_mouse` |
| [`og.input.padDevice`](input.md#fn-padDevice) | function | input | `og_input_pad_device` |
| [`og.input.rebind`](input.md#fn-rebind) | function | input | `og_input_rebind` |
| [`og.input.resetBindings`](input.md#fn-resetBindings) | function | input | `og_input_reset_bindings` |
| [`og.input.rumble`](input.md#fn-rumble) | function | input | `og_input_rumble` |
| [`og.input.saveBindings`](input.md#fn-saveBindings) | function | input | `og_input_save_bindings` |
| [`og.input.sensitivity`](input.md#fn-sensitivity) | function | input | `og_input_sensitivity` |
| [`og.input.setContexts`](input.md#fn-setContexts) | function | input | `og_input_set_contexts` |
| [`og.input.setDeadzones`](input.md#fn-setDeadzones) | function | input | `og_input_set_deadzones` |
| [`og.input.setKeyboardLayout`](input.md#fn-setKeyboardLayout) | function | input | `og_input_set_keyboard_layout` |
| [`og.input.setMouse`](input.md#fn-setMouse) | function | input | `og_input_set_mouse` |
| [`og.input.setSensitivity`](input.md#fn-setSensitivity) | function | input | `og_input_set_sensitivity` |
| [`og.input.setToggle`](input.md#fn-setToggle) | function | input | `og_input_set_toggle` |
| [`og.input.startCapture`](input.md#fn-startCapture) | function | input | `og_input_start_capture` |
| [`og.Interpolation`](render.md#record-Interpolation) | component | render | — |
| [`og.Joint`](physics.md#record-Joint) | component | physics | — |
| [`og.JointBroken`](physics.md#record-JointBroken) | event | physics | — |
| [`og.JointLimit`](anim.md#record-JointLimit) | component | anim | — |
| [`og.KeyDown`](input.md#record-KeyDown) | event | input | — |
| [`og.KeyUp`](input.md#record-KeyUp) | event | input | — |
| [`og.Light2D`](render.md#record-Light2D) | component | render | — |
| [`og.Lighting2D`](render.md#record-Lighting2D) | resource | render | — |
| [`og.LightVolume`](render.md#record-LightVolume) | component | render | — |
| [`og.Lines`](render.md#record-Lines) | component | render | — |
| [`og.load.animation`](load.md#fn-animation) | function | load | `og_load_animation` |
| [`og.load.bundle`](load.md#fn-bundle) | function | load | `og_load_bundle` |
| [`og.load.bundleInfo`](load.md#fn-bundleInfo) | function | load | `og_load_bundle_info` |
| [`og.load.defaultBundle`](load.md#fn-defaultBundle) | function | load | `og_load_default_bundle` |
| [`og.load.document`](load.md#fn-document) | function | load | `og_load_document` |
| [`og.load.emitter`](load.md#fn-emitter) | function | load | `og_load_emitter` |
| [`og.load.environment`](load.md#fn-environment) | function | load | `og_load_environment` |
| [`og.load.error`](load.md#fn-error) | function | load | `og_load_error` |
| [`og.load.flipbook`](load.md#fn-flipbook) | function | load | `og_load_flipbook` |
| [`og.load.font`](load.md#fn-font) | function | load | `og_load_font` |
| [`og.load.fontInfo`](load.md#fn-fontInfo) | function | load | `og_load_font_info` |
| [`og.load.gltf`](load.md#fn-gltf) | function | load | `og_load_gltf` |
| [`og.load.gltfInfo`](load.md#fn-gltfInfo) | function | load | `og_load_gltf_info` |
| [`og.load.look`](load.md#fn-look) | function | load | `og_load_look` |
| [`og.load.lookInfo`](load.md#fn-lookInfo) | function | load | `og_load_look_info` |
| [`og.load.lut`](load.md#fn-lut) | function | load | `og_load_lut` |
| [`og.load.palette`](load.md#fn-palette) | function | load | `og_load_palette` |
| [`og.load.post`](load.md#fn-post) | function | load | `og_load_post` |
| [`og.load.prefab`](load.md#fn-prefab) | function | load | `og_load_prefab` |
| [`og.load.rig`](load.md#fn-rig) | function | load | `og_load_rig` |
| [`og.load.rig2d`](load.md#fn-rig2d) | function | load | `og_load_rig2d` |
| [`og.load.scene`](load.md#fn-scene) | function | load | `og_load_scene` |
| [`og.load.shader`](load.md#fn-shader) | function | load | `og_load_shader` |
| [`og.load.sound`](load.md#fn-sound) | function | load | `og_load_sound` |
| [`og.load.soundInfo`](load.md#fn-soundInfo) | function | load | `og_load_sound_info` |
| [`og.load.status`](load.md#fn-status) | function | load | `og_load_status` |
| [`og.load.style`](load.md#fn-style) | function | load | `og_load_style` |
| [`og.load.texture`](load.md#fn-texture) | function | load | `og_load_texture` |
| [`og.load.textureArray`](load.md#fn-textureArray) | function | load | `og_load_texture_array` |
| [`og.load.tilemap`](load.md#fn-tilemap) | function | load | `og_load_tilemap` |
| [`og.load.timeline`](load.md#fn-timeline) | function | load | `og_load_timeline` |
| [`og.load.uiMaterial`](load.md#fn-uiMaterial) | function | load | `og_load_ui_material` |
| [`og.load.vector`](load.md#fn-vector) | function | load | `og_load_vector` |
| [`og.load.video`](load.md#fn-video) | function | load | `og_load_video` |
| [`og.load.videoInfo`](load.md#fn-videoInfo) | function | load | `og_load_video_info` |
| [`og.Lockstep`](net.md#record-Lockstep) | resource | net | — |
| [`og.LookAt`](anim.md#record-LookAt) | component | anim | — |
| [`og.MaterialStyle`](render.md#record-MaterialStyle) | resource | render | — |
| [`og.MatrixWorld`](transform.md#record-MatrixWorld) | component | transform | — |
| [`og.Mesh`](render.md#record-Mesh) | component | render | — |
| [`og.Mirror`](render.md#record-Mirror) | component | render | — |
| [`og.Model`](load.md#record-Model) | component | load | — |
| [`og.Music`](audio.md#record-Music) | component | audio | — |
| [`og.MusicBeat`](audio.md#record-MusicBeat) | event | audio | — |
| [`og.nav.build`](nav.md#fn-build) | function | nav | `og_nav_build` |
| [`og.nav.carve`](nav.md#fn-carve) | function | nav | `og_nav_carve` |
| [`og.nav.cellAt`](nav.md#fn-cellAt) | function | nav | `og_nav_cell_at` |
| [`og.nav.findPath`](nav.md#fn-findPath) | function | nav | `og_nav_find_path` |
| [`og.nav.finishLink`](nav.md#fn-finishLink) | function | nav | `og_nav_finish_link` |
| [`og.nav.flowAt`](nav.md#fn-flowAt) | function | nav | `og_nav_flow_at` |
| [`og.nav.flowField`](nav.md#fn-flowField) | function | nav | `og_nav_flow_field` |
| [`og.nav.grid`](nav.md#fn-grid) | function | nav | `og_nav_grid` |
| [`og.nav.gridFromTilemap`](nav.md#fn-gridFromTilemap) | function | nav | `og_nav_grid_from_tilemap` |
| [`og.nav.load`](nav.md#fn-load) | function | nav | `og_nav_load` |
| [`og.nav.nearest`](nav.md#fn-nearest) | function | nav | `og_nav_nearest` |
| [`og.nav.pathOf`](nav.md#fn-pathOf) | function | nav | `og_nav_path_of` |
| [`og.nav.setArea`](nav.md#fn-setArea) | function | nav | `og_nav_set_area` |
| [`og.nav.setCost`](nav.md#fn-setCost) | function | nav | `og_nav_set_cost` |
| [`og.nav.setFlowTargets`](nav.md#fn-setFlowTargets) | function | nav | `og_nav_set_flow_targets` |
| [`og.nav.setLink`](nav.md#fn-setLink) | function | nav | `og_nav_set_link` |
| [`og.nav.setLinkOpen`](nav.md#fn-setLinkOpen) | function | nav | `og_nav_set_link_open` |
| [`og.nav.setObstacle`](nav.md#fn-setObstacle) | function | nav | `og_nav_set_obstacle` |
| [`og.NavAgent`](nav.md#record-NavAgent) | component | nav | — |
| [`og.NavFilter`](nav.md#record-NavFilter) | component | nav | — |
| [`og.NavLinkReached`](nav.md#record-NavLinkReached) | event | nav | — |
| [`og.net.finalHash`](net.md#fn-finalHash) | function | net | `og_net_final_hash` |
| [`og.net.hash`](net.md#fn-hash) | function | net | `og_net_hash` |
| [`og.net.resimulating`](net.md#fn-resimulating) | function | net | `og_net_resimulating` |
| [`og.net.status`](net.md#fn-status) | function | net | `og_net_status` |
| [`og.net.submit`](net.md#fn-submit) | function | net | `og_net_submit` |
| [`og.Occluder2D`](render.md#record-Occluder2D) | component | render | — |
| [`og.OrbitCamera`](camera.md#record-OrbitCamera) | component | camera | — |
| [`og.Parent`](transform.md#record-Parent) | component | transform | — |
| [`og.ParticleSystem`](render.md#record-ParticleSystem) | component | render | — |
| [`og.path.create`](path.md#fn-create) | function | path | `og_path_create` |
| [`og.path.find`](path.md#fn-find) | function | path | `og_path_find` |
| [`og.path.length`](path.md#fn-length) | function | path | `og_path_length` |
| [`og.path.nearest`](path.md#fn-nearest) | function | path | `og_path_nearest` |
| [`og.path.sample`](path.md#fn-sample) | function | path | `og_path_sample` |
| [`og.Path2`](shapes.md#value-Path2) | value | shapes | — |
| [`og.PathFollower`](path.md#record-PathFollower) | component | path | — |
| [`og.PeerStalled`](net.md#record-PeerStalled) | event | net | — |
| [`og.Perceivable`](ai.md#record-Perceivable) | component | ai | — |
| [`og.Perception`](ai.md#record-Perception) | component | ai | — |
| [`og.Physics`](physics.md#record-Physics) | resource | physics | — |
| [`og.physics.applyForce`](physics.md#fn-applyForce) | function | physics | `og_physics_apply_force` |
| [`og.physics.applyImpulse`](physics.md#fn-applyImpulse) | function | physics | `og_physics_apply_impulse` |
| [`og.physics.applyTorque`](physics.md#fn-applyTorque) | function | physics | `og_physics_apply_torque` |
| [`og.physics.applyTorqueImpulse`](physics.md#fn-applyTorqueImpulse) | function | physics | `og_physics_apply_torque_impulse` |
| [`og.physics.clothPoints`](physics.md#fn-clothPoints) | function | physics | `og_physics_cloth_points` |
| [`og.physics.fluid3dParticles`](physics.md#fn-fluid3dParticles) | function | physics | `og_physics_fluid3d_particles` |
| [`og.physics.fluidParticles`](physics.md#fn-fluidParticles) | function | physics | `og_physics_fluid_particles` |
| [`og.physics.overlap`](physics.md#fn-overlap) | function | physics | `og_physics_overlap` |
| [`og.physics.projectPoint`](physics.md#fn-projectPoint) | function | physics | `og_physics_project_point` |
| [`og.physics.ragdollPose`](physics.md#fn-ragdollPose) | function | physics | `og_physics_ragdoll_pose` |
| [`og.physics.ragdollRigFromSkeleton`](physics.md#fn-ragdollRigFromSkeleton) | function | physics | `og_physics_ragdoll_rig_from_skeleton` |
| [`og.physics.raycast`](physics.md#fn-raycast) | function | physics | `og_physics_raycast` |
| [`og.physics.raycastAll`](physics.md#fn-raycastAll) | function | physics | `og_physics_raycast_all` |
| [`og.physics.ropePoints`](physics.md#fn-ropePoints) | function | physics | `og_physics_rope_points` |
| [`og.physics.setMesh`](physics.md#fn-setMesh) | function | physics | `og_physics_set_mesh` |
| [`og.physics.setRagdollRig`](physics.md#fn-setRagdollRig) | function | physics | `og_physics_set_ragdoll_rig` |
| [`og.physics.shapeCast`](physics.md#fn-shapeCast) | function | physics | `og_physics_shape_cast` |
| [`og.physics.softBodyPoints`](physics.md#fn-softBodyPoints) | function | physics | `og_physics_soft_body_points` |
| [`og.physics.stats`](physics.md#fn-stats) | function | physics | `og_physics_stats` |
| [`og.physics.vehicleWheels`](physics.md#fn-vehicleWheels) | function | physics | `og_physics_vehicle_wheels` |
| [`og.physics.waterHeight`](physics.md#fn-waterHeight) | function | physics | `og_physics_water_height` |
| [`og.platform.aiMark`](platform.md#fn-aiMark) | function | platform | `og_platform_ai_mark` |
| [`og.platform.info`](platform.md#fn-info) | function | platform | `og_platform_info` |
| [`og.platform.leaderboardAround`](platform.md#fn-leaderboardAround) | function | platform | `og_platform_leaderboard_around` |
| [`og.platform.leaderboardSubmit`](platform.md#fn-leaderboardSubmit) | function | platform | `og_platform_leaderboard_submit` |
| [`og.platform.leaderboardTop`](platform.md#fn-leaderboardTop) | function | platform | `og_platform_leaderboard_top` |
| [`og.platform.loadSettings`](platform.md#fn-loadSettings) | function | platform | `og_platform_load_settings` |
| [`og.platform.loadSlot`](platform.md#fn-loadSlot) | function | platform | `og_platform_load_slot` |
| [`og.platform.now`](platform.md#fn-now) | function | platform | `og_platform_now` |
| [`og.platform.point`](platform.md#fn-point) | function | platform | `og_platform_point` |
| [`og.platform.result`](platform.md#fn-result) | function | platform | `og_platform_result` |
| [`og.platform.roomCreate`](platform.md#fn-roomCreate) | function | platform | `og_platform_room_create` |
| [`og.platform.roomInfo`](platform.md#fn-roomInfo) | function | platform | `og_platform_room_info` |
| [`og.platform.roomJoin`](platform.md#fn-roomJoin) | function | platform | `og_platform_room_join` |
| [`og.platform.roomLeave`](platform.md#fn-roomLeave) | function | platform | `og_platform_room_leave` |
| [`og.platform.roomMessage`](platform.md#fn-roomMessage) | function | platform | `og_platform_room_message` |
| [`og.platform.roomPeers`](platform.md#fn-roomPeers) | function | platform | `og_platform_room_peers` |
| [`og.platform.roomSend`](platform.md#fn-roomSend) | function | platform | `og_platform_room_send` |
| [`og.platform.saveSettings`](platform.md#fn-saveSettings) | function | platform | `og_platform_save_settings` |
| [`og.platform.saveSlot`](platform.md#fn-saveSlot) | function | platform | `og_platform_save_slot` |
| [`og.platform.storeGet`](platform.md#fn-storeGet) | function | platform | `og_platform_store_get` |
| [`og.platform.storeList`](platform.md#fn-storeList) | function | platform | `og_platform_store_list` |
| [`og.platform.storeSet`](platform.md#fn-storeSet) | function | platform | `og_platform_store_set` |
| [`og.platform.telemetry`](platform.md#fn-telemetry) | function | platform | `og_platform_telemetry` |
| [`og.PlatformAnswered`](platform.md#record-PlatformAnswered) | event | platform | — |
| [`og.PlatformerController`](physics.md#record-PlatformerController) | component | physics | — |
| [`og.PlayerInput`](net.md#record-PlayerInput) | component | net | — |
| [`og.PointerDown`](input.md#record-PointerDown) | event | input | — |
| [`og.PointerLock`](input.md#record-PointerLock) | resource | input | — |
| [`og.PointerMove`](input.md#record-PointerMove) | event | input | — |
| [`og.Pointers`](input.md#record-Pointers) | resource | input | — |
| [`og.PointerUp`](input.md#record-PointerUp) | event | input | — |
| [`og.PointLight`](render.md#record-PointLight) | component | render | — |
| [`og.Points`](render.md#record-Points) | component | render | — |
| [`og.PostEffects`](render.md#record-PostEffects) | resource | render | — |
| [`og.PrefabLink`](load.md#record-PrefabLink) | component | load | — |
| [`og.procgen.bsp`](procgen.md#fn-bsp) | function | procgen | `og_procgen_bsp` |
| [`og.procgen.noise`](procgen.md#fn-noise) | function | procgen | `og_procgen_noise` |
| [`og.procgen.noiseGrid`](procgen.md#fn-noiseGrid) | function | procgen | `og_procgen_noise_grid` |
| [`og.procgen.poissonDisk`](procgen.md#fn-poissonDisk) | function | procgen | `og_procgen_poisson_disk` |
| [`og.procgen.wfc`](procgen.md#fn-wfc) | function | procgen | `og_procgen_wfc` |
| [`og.Ragdoll`](physics.md#record-Ragdoll) | component | physics | — |
| [`og.RectLight`](render.md#record-RectLight) | component | render | — |
| [`og.ReflectionProbe`](render.md#record-ReflectionProbe) | component | render | — |
| [`og.render.applyLook`](render.md#fn-applyLook) | function | render | `og_render_apply_look` |
| [`og.render.applyQuality`](render.md#fn-applyQuality) | function | render | `og_render_apply_quality` |
| [`og.render.bakedLighting`](render.md#fn-bakedLighting) | function | render | `og_render_baked_lighting` |
| [`og.render.boxGeometry`](render.md#fn-boxGeometry) | function | render | `og_render_box_geometry` |
| [`og.render.bufferGeometry`](render.md#fn-bufferGeometry) | function | render | `og_render_buffer_geometry` |
| [`og.render.capabilities`](render.md#fn-capabilities) | function | render | `og_render_capabilities` |
| [`og.render.capsuleGeometry`](render.md#fn-capsuleGeometry) | function | render | `og_render_capsule_geometry` |
| [`og.render.circleGeometry`](render.md#fn-circleGeometry) | function | render | `og_render_circle_geometry` |
| [`og.render.coneGeometry`](render.md#fn-coneGeometry) | function | render | `og_render_cone_geometry` |
| [`og.render.csgGeometry`](render.md#fn-csgGeometry) | function | render | `og_render_csg_geometry` |
| [`og.render.cylinderGeometry`](render.md#fn-cylinderGeometry) | function | render | `og_render_cylinder_geometry` |
| [`og.render.display`](render.md#fn-display) | function | render | `og_render_display` |
| [`og.render.dodecahedronGeometry`](render.md#fn-dodecahedronGeometry) | function | render | `og_render_dodecahedron_geometry` |
| [`og.render.extrudeGeometry`](render.md#fn-extrudeGeometry) | function | render | `og_render_extrude_geometry` |
| [`og.render.floatingText`](render.md#fn-floatingText) | function | render | `og_render_floating_text` |
| [`og.render.geometryInfo`](render.md#fn-geometryInfo) | function | render | `og_render_geometry_info` |
| [`og.render.icosahedronGeometry`](render.md#fn-icosahedronGeometry) | function | render | `og_render_icosahedron_geometry` |
| [`og.render.latheGeometry`](render.md#fn-latheGeometry) | function | render | `og_render_lathe_geometry` |
| [`og.render.lines`](render.md#fn-lines) | function | render | `og_render_lines` |
| [`og.render.look`](render.md#fn-look) | function | render | `og_render_look` |
| [`og.render.measureText`](render.md#fn-measureText) | function | render | `og_render_measure_text` |
| [`og.render.mergeGeometries`](render.md#fn-mergeGeometries) | function | render | `og_render_merge_geometries` |
| [`og.render.octahedronGeometry`](render.md#fn-octahedronGeometry) | function | render | `og_render_octahedron_geometry` |
| [`og.render.planeGeometry`](render.md#fn-planeGeometry) | function | render | `og_render_plane_geometry` |
| [`og.render.points`](render.md#fn-points) | function | render | `og_render_points` |
| [`og.render.polyhedronGeometry`](render.md#fn-polyhedronGeometry) | function | render | `og_render_polyhedron_geometry` |
| [`og.render.postErrors`](render.md#fn-postErrors) | function | render | `og_render_post_errors` |
| [`og.render.raycast`](render.md#fn-raycast) | function | render | `og_render_raycast` |
| [`og.render.renderTarget`](render.md#fn-renderTarget) | function | render | `og_render_render_target` |
| [`og.render.resetDisplay`](render.md#fn-resetDisplay) | function | render | `og_render_reset_display` |
| [`og.render.ribbon`](render.md#fn-ribbon) | function | render | `og_render_ribbon` |
| [`og.render.ringGeometry`](render.md#fn-ringGeometry) | function | render | `og_render_ring_geometry` |
| [`og.render.setDisplay`](render.md#fn-setDisplay) | function | render | `og_render_set_display` |
| [`og.render.setMaterialShader`](render.md#fn-setMaterialShader) | function | render | `og_render_set_material_shader` |
| [`og.render.setPostParam`](render.md#fn-setPostParam) | function | render | `og_render_set_post_param` |
| [`og.render.setShaderParam`](render.md#fn-setShaderParam) | function | render | `og_render_set_shader_param` |
| [`og.render.shaderMaterial`](render.md#fn-shaderMaterial) | function | render | `og_render_shader_material` |
| [`og.render.shapeGeometry`](render.md#fn-shapeGeometry) | function | render | `og_render_shape_geometry` |
| [`og.render.snap`](render.md#fn-snap) | function | render | `og_render_snap` |
| [`og.render.snapCamera`](render.md#fn-snapCamera) | function | render | `og_render_snap_camera` |
| [`og.render.sphereGeometry`](render.md#fn-sphereGeometry) | function | render | `og_render_sphere_geometry` |
| [`og.render.sprite`](render.md#fn-sprite) | function | render | `og_render_sprite` |
| [`og.render.standardMaterial`](render.md#fn-standardMaterial) | function | render | `og_render_standard_material` |
| [`og.render.sweepGeometry`](render.md#fn-sweepGeometry) | function | render | `og_render_sweep_geometry` |
| [`og.render.tetrahedronGeometry`](render.md#fn-tetrahedronGeometry) | function | render | `og_render_tetrahedron_geometry` |
| [`og.render.text`](render.md#fn-text) | function | render | `og_render_text` |
| [`og.render.torusGeometry`](render.md#fn-torusGeometry) | function | render | `og_render_torus_geometry` |
| [`og.render.tubeGeometry`](render.md#fn-tubeGeometry) | function | render | `og_render_tube_geometry` |
| [`og.render.updateGeometry`](render.md#fn-updateGeometry) | function | render | `og_render_update_geometry` |
| [`og.render.updateMaterial`](render.md#fn-updateMaterial) | function | render | `og_render_update_material` |
| [`og.RenderSettings`](render.md#record-RenderSettings) | resource | render | — |
| [`og.ReverbZone`](audio.md#record-ReverbZone) | component | audio | — |
| [`og.RigidBody`](physics.md#record-RigidBody) | component | physics | — |
| [`og.Rng`](rng.md#record-Rng) | resource | rng | — |
| [`og.RoomJoined`](platform.md#record-RoomJoined) | event | platform | — |
| [`og.RoomMessage`](platform.md#record-RoomMessage) | event | platform | — |
| [`og.RoomPeerJoined`](platform.md#record-RoomPeerJoined) | event | platform | — |
| [`og.RoomPeerLeft`](platform.md#record-RoomPeerLeft) | event | platform | — |
| [`og.Rope`](physics.md#record-Rope) | component | physics | — |
| [`og.SeatDropped`](net.md#record-SeatDropped) | event | net | — |
| [`og.SeatRejoined`](net.md#record-SeatRejoined) | event | net | — |
| [`og.SensorEntered`](physics.md#record-SensorEntered) | event | physics | — |
| [`og.SensorExited`](physics.md#record-SensorExited) | event | physics | — |
| [`og.Shape`](shapes.md#value-Shape) | value | shapes | — |
| [`og.Skeleton`](anim.md#record-Skeleton) | component | anim | — |
| [`og.Skeleton2D`](anim.md#record-Skeleton2D) | component | anim | — |
| [`og.Skeleton2DEvent`](anim.md#record-Skeleton2DEvent) | event | anim | — |
| [`og.Sky`](atmosphere.md#record-Sky) | resource | atmosphere | — |
| [`og.SoftBody`](physics.md#record-SoftBody) | component | physics | — |
| [`og.SoundEnded`](audio.md#record-SoundEnded) | event | audio | — |
| [`og.SoundStarted`](audio.md#record-SoundStarted) | event | audio | — |
| [`og.Splash`](physics.md#record-Splash) | event | physics | — |
| [`og.SpotLight`](render.md#record-SpotLight) | component | render | — |
| [`og.SpringBone`](anim.md#record-SpringBone) | component | anim | — |
| [`og.SpringCollider`](anim.md#record-SpringCollider) | component | anim | — |
| [`og.Sprite`](render.md#record-Sprite) | component | render | — |
| [`og.Steering`](nav.md#record-Steering) | component | nav | — |
| [`og.stream.budget`](stream.md#fn-budget) | function | stream | `og_stream_budget` |
| [`og.stream.release`](stream.md#fn-release) | function | stream | `og_stream_release` |
| [`og.stream.request`](stream.md#fn-request) | function | stream | `og_stream_request` |
| [`og.stream.setBudget`](stream.md#fn-setBudget) | function | stream | `og_stream_set_budget` |
| [`og.stream.status`](stream.md#fn-status) | function | stream | `og_stream_status` |
| [`og.StreamEvent`](stream.md#record-StreamEvent) | event | stream | — |
| [`og.Stylise`](render.md#record-Stylise) | resource | render | — |
| [`og.systems.cameras`](systems.md#value-cameras) | value | systems | — |
| [`og.Terrain`](terrain.md#record-Terrain) | component | terrain | — |
| [`og.terrain.bounds`](terrain.md#fn-bounds) | function | terrain | `og_terrain_bounds` |
| [`og.terrain.edit`](terrain.md#fn-edit) | function | terrain | `og_terrain_edit` |
| [`og.terrain.layer`](terrain.md#fn-layer) | function | terrain | `og_terrain_layer` |
| [`og.terrain.load`](terrain.md#fn-load) | function | terrain | `og_terrain_load` |
| [`og.terrain.sample`](terrain.md#fn-sample) | function | terrain | `og_terrain_sample` |
| [`og.terrain.setLayer`](terrain.md#fn-setLayer) | function | terrain | `og_terrain_set_layer` |
| [`og.TerrainScatter`](terrain.md#record-TerrainScatter) | component | terrain | — |
| [`og.TextInput`](input.md#record-TextInput) | event | input | — |
| [`og.Tilemap`](tilemap.md#record-Tilemap) | component | tilemap | — |
| [`og.tilemap.cellAt`](tilemap.md#fn-cellAt) | function | tilemap | `og_tilemap_cell_at` |
| [`og.tilemap.cellCenter`](tilemap.md#fn-cellCenter) | function | tilemap | `og_tilemap_cell_center` |
| [`og.tilemap.create`](tilemap.md#fn-create) | function | tilemap | `og_tilemap_create` |
| [`og.tilemap.fill`](tilemap.md#fn-fill) | function | tilemap | `og_tilemap_fill` |
| [`og.tilemap.getTile`](tilemap.md#fn-getTile) | function | tilemap | `og_tilemap_get_tile` |
| [`og.tilemap.neighbours`](tilemap.md#fn-neighbours) | function | tilemap | `og_tilemap_neighbours` |
| [`og.tilemap.resize`](tilemap.md#fn-resize) | function | tilemap | `og_tilemap_resize` |
| [`og.tilemap.setTile`](tilemap.md#fn-setTile) | function | tilemap | `og_tilemap_set_tile` |
| [`og.Time`](time.md#record-Time) | resource | time | — |
| [`og.time.hitStop`](time.md#fn-hitStop) | function | time | `og_time_hit_stop` |
| [`og.time.setScale`](time.md#fn-setScale) | function | time | `og_time_set_scale` |
| [`og.Timeline`](timeline.md#record-Timeline) | component | timeline | — |
| [`og.timeline.info`](timeline.md#fn-info) | function | timeline | `og_timeline_info` |
| [`og.timeline.pause`](timeline.md#fn-pause) | function | timeline | `og_timeline_pause` |
| [`og.timeline.play`](timeline.md#fn-play) | function | timeline | `og_timeline_play` |
| [`og.timeline.seek`](timeline.md#fn-seek) | function | timeline | `og_timeline_seek` |
| [`og.timeline.stop`](timeline.md#fn-stop) | function | timeline | `og_timeline_stop` |
| [`og.TimelineEnded`](timeline.md#record-TimelineEnded) | event | timeline | — |
| [`og.TimelineMarker`](timeline.md#record-TimelineMarker) | event | timeline | — |
| [`og.TimelineStarted`](timeline.md#record-TimelineStarted) | event | timeline | — |
| [`og.Trail`](render.md#record-Trail) | component | render | — |
| [`og.Transform`](transform.md#record-Transform) | component | transform | — |
| [`og.transform.compose`](transform.md#fn-compose) | function | transform | `og_transform_compose` |
| [`og.transform.decompose`](transform.md#fn-decompose) | function | transform | `og_transform_decompose` |
| [`og.transform.fromAxisAngle`](transform.md#fn-fromAxisAngle) | function | transform | `og_transform_from_axis_angle` |
| [`og.transform.lookAt`](transform.md#fn-lookAt) | function | transform | `og_transform_look_at` |
| [`og.tween.active`](tween.md#fn-active) | function | tween | `og_tween_active` |
| [`og.tween.cancel`](tween.md#fn-cancel) | function | tween | `og_tween_cancel` |
| [`og.tween.cancelEntity`](tween.md#fn-cancelEntity) | function | tween | `og_tween_cancel_entity` |
| [`og.tween.flash`](tween.md#fn-flash) | function | tween | `og_tween_flash` |
| [`og.tween.parallel`](tween.md#fn-parallel) | function | tween | `og_tween_parallel` |
| [`og.tween.sequence`](tween.md#fn-sequence) | function | tween | `og_tween_sequence` |
| [`og.tween.squash`](tween.md#fn-squash) | function | tween | `og_tween_squash` |
| [`og.tween.to`](tween.md#fn-to) | function | tween | `og_tween_to` |
| [`og.TweenCompleted`](tween.md#record-TweenCompleted) | event | tween | — |
| [`og.ui.a11y`](ui.md#fn-a11y) | function | ui | `og_ui_a11y` |
| [`og.ui.abilityBar`](ui.md#fn-abilityBar) | function | ui | — |
| [`og.ui.abilitySlot`](ui.md#fn-abilitySlot) | function | ui | — |
| [`og.ui.accordion`](ui.md#fn-accordion) | function | ui | — |
| [`og.ui.action`](ui.md#fn-action) | function | ui | `og_ui_action` |
| [`og.ui.actionBar`](ui.md#fn-actionBar) | function | ui | `og_ui_action_bar` |
| [`og.ui.actionButton`](ui.md#fn-actionButton) | function | ui | `og_ui_action_button` |
| [`og.ui.actionHeld`](ui.md#fn-actionHeld) | function | ui | `og_ui_action_held` |
| [`og.ui.actionHints`](ui.md#fn-actionHints) | function | ui | — |
| [`og.ui.alertDialog`](ui.md#fn-alertDialog) | function | ui | — |
| [`og.ui.ammoCounter`](ui.md#fn-ammoCounter) | function | ui | — |
| [`og.ui.anchor`](ui.md#fn-anchor) | function | ui | — |
| [`og.ui.animate`](ui.md#fn-animate) | function | ui | `og_ui_animate` |
| [`og.ui.artSlot`](ui.md#fn-artSlot) | function | ui | — |
| [`og.ui.back`](ui.md#fn-back) | function | ui | `og_ui_back` |
| [`og.ui.badge`](ui.md#fn-badge) | function | ui | `og_ui_badge` |
| [`og.ui.banner`](ui.md#fn-banner) | function | ui | — |
| [`og.ui.bounds`](ui.md#fn-bounds) | function | ui | `og_ui_bounds` |
| [`og.ui.breadcrumbs`](ui.md#fn-breadcrumbs) | function | ui | — |
| [`og.ui.breakpoint`](ui.md#fn-breakpoint) | function | ui | `og_ui_breakpoint` |
| [`og.ui.burst`](ui.md#fn-burst) | function | ui | `og_ui_burst` |
| [`og.ui.button`](ui.md#fn-button) | function | ui | `og_ui_button` |
| [`og.ui.calendar`](ui.md#fn-calendar) | function | ui | — |
| [`og.ui.card`](ui.md#fn-card) | function | ui | — |
| [`og.ui.carousel`](ui.md#fn-carousel) | function | ui | — |
| [`og.ui.castBar`](ui.md#fn-castBar) | function | ui | — |
| [`og.ui.celebration`](ui.md#fn-celebration) | function | ui | — |
| [`og.ui.chart`](ui.md#fn-chart) | function | ui | — |
| [`og.ui.checkbox`](ui.md#fn-checkbox) | function | ui | `og_ui_checkbox` |
| [`og.ui.checkboxList`](ui.md#fn-checkboxList) | function | ui | `og_ui_checkbox_list` |
| [`og.ui.checkField`](ui.md#fn-checkField) | function | ui | — |
| [`og.ui.chip`](ui.md#fn-chip) | function | ui | `og_ui_chip` |
| [`og.ui.clear`](ui.md#fn-clear) | function | ui | `og_ui_clear` |
| [`og.ui.close`](ui.md#fn-close) | function | ui | `og_ui_close` |
| [`og.ui.closeList`](ui.md#fn-closeList) | function | ui | `og_ui_close_list` |
| [`og.ui.closeModal`](ui.md#fn-closeModal) | function | ui | `og_ui_close_modal` |
| [`og.ui.colorPicker`](ui.md#fn-colorPicker) | function | ui | — |
| [`og.ui.column`](ui.md#fn-column) | function | ui | — |
| [`og.ui.comboCounter`](ui.md#fn-comboCounter) | function | ui | — |
| [`og.ui.compass`](ui.md#fn-compass) | function | ui | — |
| [`og.ui.component`](ui.md#fn-component) | function | ui | `og_ui_component` |
| [`og.ui.configure`](ui.md#fn-configure) | function | ui | `og_ui_configure` |
| [`og.ui.confirm`](ui.md#fn-confirm) | function | ui | — |
| [`og.ui.confirmDialog`](ui.md#fn-confirmDialog) | function | ui | — |
| [`og.ui.contextMenu`](ui.md#fn-contextMenu) | function | ui | — |
| [`og.ui.cooldown`](ui.md#fn-cooldown) | function | ui | `og_ui_cooldown` |
| [`og.ui.countBadge`](ui.md#fn-countBadge) | function | ui | — |
| [`og.ui.counter`](ui.md#fn-counter) | function | ui | `og_ui_counter` |
| [`og.ui.crosshair`](ui.md#fn-crosshair) | function | ui | — |
| [`og.ui.cutscene`](ui.md#fn-cutscene) | function | ui | — |
| [`og.ui.cycler`](ui.md#fn-cycler) | function | ui | — |
| [`og.ui.damageDirection`](ui.md#fn-damageDirection) | function | ui | — |
| [`og.ui.damageNumbers`](ui.md#fn-damageNumbers) | function | ui | — |
| [`og.ui.dataGrid`](ui.md#fn-dataGrid) | function | ui | — |
| [`og.ui.dataList`](ui.md#fn-dataList) | function | ui | — |
| [`og.ui.defineAction`](ui.md#fn-defineAction) | function | ui | `og_ui_define_action` |
| [`og.ui.defineMass`](ui.md#fn-defineMass) | function | ui | `og_ui_define_mass` |
| [`og.ui.defineTimeline`](ui.md#fn-defineTimeline) | function | ui | `og_ui_define_timeline` |
| [`og.ui.device`](ui.md#fn-device) | function | ui | `og_ui_device` |
| [`og.ui.dialog`](ui.md#fn-dialog) | function | ui | — |
| [`og.ui.dialogue`](ui.md#fn-dialogue) | function | ui | `og_ui_dialogue` |
| [`og.ui.dismissToast`](ui.md#fn-dismissToast) | function | ui | `og_ui_dismiss_toast` |
| [`og.ui.divider`](ui.md#fn-divider) | function | ui | `og_ui_divider` |
| [`og.ui.docEvents`](ui.md#fn-docEvents) | function | ui | `og_ui_doc_events` |
| [`og.ui.documentSchema`](ui.md#fn-documentSchema) | function | ui | `og_ui_document_schema` |
| [`og.ui.dpad`](ui.md#fn-dpad) | function | ui | `og_ui_dpad` |
| [`og.ui.dragging`](ui.md#fn-dragging) | function | ui | `og_ui_dragging` |
| [`og.ui.dropdown`](ui.md#fn-dropdown) | function | ui | — |
| [`og.ui.effectLayer`](ui.md#fn-effectLayer) | function | ui | — |
| [`og.ui.emitter`](ui.md#fn-emitter) | function | ui | — |
| [`og.ui.feedback`](ui.md#fn-feedback) | function | ui | `og_ui_feedback` |
| [`og.ui.flightHud`](ui.md#fn-flightHud) | function | ui | — |
| [`og.ui.focus`](ui.md#fn-focus) | function | ui | `og_ui_focus` |
| [`og.ui.forceState`](ui.md#fn-forceState) | function | ui | `og_ui_force_state` |
| [`og.ui.formatNumber`](ui.md#fn-formatNumber) | function | ui | — |
| [`og.ui.frameDirection`](ui.md#fn-frameDirection) | function | ui | `og_ui_frame_direction` |
| [`og.ui.frameGlyph`](ui.md#fn-frameGlyph) | function | ui | `og_ui_frame_glyph` |
| [`og.ui.framePointer`](ui.md#fn-framePointer) | function | ui | `og_ui_frame_pointer` |
| [`og.ui.frameTheme`](ui.md#fn-frameTheme) | function | ui | `og_ui_frame_theme` |
| [`og.ui.frameViewport`](ui.md#fn-frameViewport) | function | ui | `og_ui_frame_viewport` |
| [`og.ui.gameInput`](ui.md#fn-gameInput) | function | ui | `og_ui_game_input` |
| [`og.ui.gesture`](ui.md#fn-gesture) | function | ui | `og_ui_gesture` |
| [`og.ui.glyph`](ui.md#fn-glyph) | function | ui | `og_ui_glyph` |
| [`og.ui.grid`](ui.md#fn-grid) | function | ui | — |
| [`og.ui.gridItem`](ui.md#fn-gridItem) | function | ui | `og_ui_grid_item` |
| [`og.ui.hasIcon`](ui.md#fn-hasIcon) | function | ui | `og_ui_has_icon` |
| [`og.ui.holdButton`](ui.md#fn-holdButton) | function | ui | — |
| [`og.ui.hudTimer`](ui.md#fn-hudTimer) | function | ui | — |
| [`og.ui.icon`](ui.md#fn-icon) | function | ui | `og_ui_icon` |
| [`og.ui.iconButton`](ui.md#fn-iconButton) | function | ui | — |
| [`og.ui.iconMark`](ui.md#fn-iconMark) | function | ui | — |
| [`og.ui.idi`](ui.md#fn-idi) | function | ui | — |
| [`og.ui.idleHud`](ui.md#fn-idleHud) | function | ui | — |
| [`og.ui.image`](ui.md#fn-image) | function | ui | `og_ui_image` |
| [`og.ui.inputArea`](ui.md#fn-inputArea) | function | ui | — |
| [`og.ui.inputField`](ui.md#fn-inputField) | function | ui | — |
| [`og.ui.interactionPrompt`](ui.md#fn-interactionPrompt) | function | ui | — |
| [`og.ui.inventoryScreen`](ui.md#fn-inventoryScreen) | function | ui | — |
| [`og.ui.joystick`](ui.md#fn-joystick) | function | ui | `og_ui_joystick` |
| [`og.ui.judgement`](ui.md#fn-judgement) | function | ui | — |
| [`og.ui.keyBindingRow`](ui.md#fn-keyBindingRow) | function | ui | — |
| [`og.ui.keyValue`](ui.md#fn-keyValue) | function | ui | `og_ui_key_value` |
| [`og.ui.keyValueRows`](ui.md#fn-keyValueRows) | function | ui | — |
| [`og.ui.killfeed`](ui.md#fn-killfeed) | function | ui | — |
| [`og.ui.label`](ui.md#fn-label) | function | ui | `og_ui_label` |
| [`og.ui.lastPath`](ui.md#fn-lastPath) | function | ui | `og_ui_last_path` |
| [`og.ui.layers`](ui.md#fn-layers) | function | ui | `og_ui_layers` |
| [`og.ui.layoutStats`](ui.md#fn-layoutStats) | function | ui | `og_ui_layout_stats` |
| [`og.ui.leaderboardScreen`](ui.md#fn-leaderboardScreen) | function | ui | — |
| [`og.ui.levelSelect`](ui.md#fn-levelSelect) | function | ui | — |
| [`og.ui.list`](ui.md#fn-list) | function | ui | — |
| [`og.ui.listItem`](ui.md#fn-listItem) | function | ui | `og_ui_list_item` |
| [`og.ui.loadingScreen`](ui.md#fn-loadingScreen) | function | ui | — |
| [`og.ui.lootLabel`](ui.md#fn-lootLabel) | function | ui | — |
| [`og.ui.lowHealthVignette`](ui.md#fn-lowHealthVignette) | function | ui | — |
| [`og.ui.mainMenu`](ui.md#fn-mainMenu) | function | ui | — |
| [`og.ui.massCount`](ui.md#fn-massCount) | function | ui | `og_ui_mass_count` |
| [`og.ui.memory`](ui.md#fn-memory) | function | ui | `og_ui_memory` |
| [`og.ui.menu`](ui.md#fn-menu) | function | ui | — |
| [`og.ui.meta`](ui.md#fn-meta) | function | ui | `og_ui_meta` |
| [`og.ui.meter`](ui.md#fn-meter) | function | ui | `og_ui_meter` |
| [`og.ui.minimap`](ui.md#fn-minimap) | function | ui | `og_ui_minimap` |
| [`og.ui.minimapView`](ui.md#fn-minimapView) | function | ui | — |
| [`og.ui.modal`](ui.md#fn-modal) | function | ui | — |
| [`og.ui.motionEvents`](ui.md#fn-motionEvents) | function | ui | `og_ui_motion_events` |
| [`og.ui.nameplate`](ui.md#fn-nameplate) | function | ui | — |
| [`og.ui.navigate`](ui.md#fn-navigate) | function | ui | `og_ui_navigate` |
| [`og.ui.nodeGraph`](ui.md#fn-nodeGraph) | function | ui | — |
| [`og.ui.notifications`](ui.md#fn-notifications) | function | ui | — |
| [`og.ui.numberCounter`](ui.md#fn-numberCounter) | function | ui | — |
| [`og.ui.numberStepper`](ui.md#fn-numberStepper) | function | ui | — |
| [`og.ui.objectiveTracker`](ui.md#fn-objectiveTracker) | function | ui | — |
| [`og.ui.onScreenKeyboard`](ui.md#fn-onScreenKeyboard) | function | ui | — |
| [`og.ui.open`](ui.md#fn-open) | function | ui | `og_ui_open` |
| [`og.ui.openAnchor`](ui.md#fn-openAnchor) | function | ui | `og_ui_open_anchor` |
| [`og.ui.openGrid`](ui.md#fn-openGrid) | function | ui | `og_ui_open_grid` |
| [`og.ui.openList`](ui.md#fn-openList) | function | ui | `og_ui_open_list` |
| [`og.ui.openModal`](ui.md#fn-openModal) | function | ui | `og_ui_open_modal` |
| [`og.ui.openPanel`](ui.md#fn-openPanel) | function | ui | `og_ui_open_panel` |
| [`og.ui.openPath`](ui.md#fn-openPath) | function | ui | `og_ui_open_path` |
| [`og.ui.openScroll`](ui.md#fn-openScroll) | function | ui | `og_ui_open_scroll` |
| [`og.ui.openSlot`](ui.md#fn-openSlot) | function | ui | `og_ui_open_slot` |
| [`og.ui.openTransition`](ui.md#fn-openTransition) | function | ui | `og_ui_open_transition` |
| [`og.ui.openWorldAnchor`](ui.md#fn-openWorldAnchor) | function | ui | `og_ui_open_world_anchor` |
| [`og.ui.openWorldPanel`](ui.md#fn-openWorldPanel) | function | ui | `og_ui_open_world_panel` |
| [`og.ui.orbMeter`](ui.md#fn-orbMeter) | function | ui | — |
| [`og.ui.overheadBars`](ui.md#fn-overheadBars) | function | ui | — |
| [`og.ui.overlaps`](ui.md#fn-overlaps) | function | ui | `og_ui_overlaps` |
| [`og.ui.pagination`](ui.md#fn-pagination) | function | ui | — |
| [`og.ui.panel`](ui.md#fn-panel) | function | ui | — |
| [`og.ui.particles`](ui.md#fn-particles) | function | ui | `og_ui_particles` |
| [`og.ui.partyFrames`](ui.md#fn-partyFrames) | function | ui | — |
| [`og.ui.pathLine`](ui.md#fn-pathLine) | function | ui | — |
| [`og.ui.pause`](ui.md#fn-pause) | function | ui | `og_ui_pause` |
| [`og.ui.paused`](ui.md#fn-paused) | function | ui | `og_ui_paused` |
| [`og.ui.pauseMenu`](ui.md#fn-pauseMenu) | function | ui | — |
| [`og.ui.placementGhost`](ui.md#fn-placementGhost) | function | ui | — |
| [`og.ui.platformerHud`](ui.md#fn-platformerHud) | function | ui | — |
| [`og.ui.play`](ui.md#fn-play) | function | ui | `og_ui_play` |
| [`og.ui.playInfo`](ui.md#fn-playInfo) | function | ui | `og_ui_play_info` |
| [`og.ui.playTimeline`](ui.md#fn-playTimeline) | function | ui | `og_ui_play_timeline` |
| [`og.ui.pop`](ui.md#fn-pop) | function | ui | `og_ui_pop` |
| [`og.ui.popover`](ui.md#fn-popover) | function | ui | — |
| [`og.ui.previewViewport`](ui.md#fn-previewViewport) | function | ui | `og_ui_preview_viewport` |
| [`og.ui.progress`](ui.md#fn-progress) | function | ui | `og_ui_progress` |
| [`og.ui.progressBar`](ui.md#fn-progressBar) | function | ui | — |
| [`og.ui.progressRing`](ui.md#fn-progressRing) | function | ui | — |
| [`og.ui.props`](ui.md#fn-props) | function | ui | `og_ui_props` |
| [`og.ui.prose`](ui.md#fn-prose) | function | ui | — |
| [`og.ui.push`](ui.md#fn-push) | function | ui | `og_ui_push` |
| [`og.ui.pushButton`](ui.md#fn-pushButton) | function | ui | — |
| [`og.ui.pushMass`](ui.md#fn-pushMass) | function | ui | `og_ui_push_mass` |
| [`og.ui.puzzleHud`](ui.md#fn-puzzleHud) | function | ui | — |
| [`og.ui.racingHud`](ui.md#fn-racingHud) | function | ui | — |
| [`og.ui.radialMenu`](ui.md#fn-radialMenu) | function | ui | — |
| [`og.ui.radialMeter`](ui.md#fn-radialMeter) | function | ui | — |
| [`og.ui.radio`](ui.md#fn-radio) | function | ui | `og_ui_radio` |
| [`og.ui.radioGroup`](ui.md#fn-radioGroup) | function | ui | — |
| [`og.ui.raidFrames`](ui.md#fn-raidFrames) | function | ui | — |
| [`og.ui.rangeSlider`](ui.md#fn-rangeSlider) | function | ui | — |
| [`og.ui.remember`](ui.md#fn-remember) | function | ui | `og_ui_remember` |
| [`og.ui.replace`](ui.md#fn-replace) | function | ui | `og_ui_replace` |
| [`og.ui.resourceBar`](ui.md#fn-resourceBar) | function | ui | — |
| [`og.ui.resultsScreen`](ui.md#fn-resultsScreen) | function | ui | — |
| [`og.ui.resume`](ui.md#fn-resume) | function | ui | `og_ui_resume` |
| [`og.ui.reverse`](ui.md#fn-reverse) | function | ui | `og_ui_reverse` |
| [`og.ui.ribbon`](ui.md#fn-ribbon) | function | ui | — |
| [`og.ui.richText`](ui.md#fn-richText) | function | ui | `og_ui_rich_text` |
| [`og.ui.row`](ui.md#fn-row) | function | ui | — |
| [`og.ui.saveEdits`](ui.md#fn-saveEdits) | function | ui | `og_ui_save_edits` |
| [`og.ui.sceneViewport`](ui.md#fn-sceneViewport) | function | ui | `og_ui_scene_viewport` |
| [`og.ui.scoreboard`](ui.md#fn-scoreboard) | function | ui | — |
| [`og.ui.scorebug`](ui.md#fn-scorebug) | function | ui | — |
| [`og.ui.screen`](ui.md#fn-screen) | function | ui | `og_ui_screen` |
| [`og.ui.screenTransition`](ui.md#fn-screenTransition) | function | ui | `og_ui_screen_transition` |
| [`og.ui.screenVm`](ui.md#fn-screenVm) | function | ui | `og_ui_screen_vm` |
| [`og.ui.scroll`](ui.md#fn-scroll) | function | ui | — |
| [`og.ui.scrollArea`](ui.md#fn-scrollArea) | function | ui | — |
| [`og.ui.searchBox`](ui.md#fn-searchBox) | function | ui | — |
| [`og.ui.seconds`](ui.md#fn-seconds) | function | ui | `og_ui_seconds` |
| [`og.ui.seek`](ui.md#fn-seek) | function | ui | `og_ui_seek` |
| [`og.ui.segmented`](ui.md#fn-segmented) | function | ui | — |
| [`og.ui.select`](ui.md#fn-select) | function | ui | `og_ui_select` |
| [`og.ui.selectionRing`](ui.md#fn-selectionRing) | function | ui | — |
| [`og.ui.semantics`](ui.md#fn-semantics) | function | ui | `og_ui_semantics` |
| [`og.ui.separator`](ui.md#fn-separator) | function | ui | — |
| [`og.ui.setDevice`](ui.md#fn-setDevice) | function | ui | `og_ui_set_device` |
| [`og.ui.setDraw`](ui.md#fn-setDraw) | function | ui | `og_ui_set_draw` |
| [`og.ui.setFocus`](ui.md#fn-setFocus) | function | ui | `og_ui_set_focus` |
| [`og.ui.setHaptics`](ui.md#fn-setHaptics) | function | ui | `og_ui_set_haptics` |
| [`og.ui.setNavigation`](ui.md#fn-setNavigation) | function | ui | `og_ui_set_navigation` |
| [`og.ui.setPlayers`](ui.md#fn-setPlayers) | function | ui | `og_ui_set_players` |
| [`og.ui.setProp`](ui.md#fn-setProp) | function | ui | `og_ui_set_prop` |
| [`og.ui.settingsScreen`](ui.md#fn-settingsScreen) | function | ui | — |
| [`og.ui.shake`](ui.md#fn-shake) | function | ui | `og_ui_shake` |
| [`og.ui.shapeBox`](ui.md#fn-shapeBox) | function | ui | — |
| [`og.ui.sheet`](ui.md#fn-sheet) | function | ui | — |
| [`og.ui.shooterHud`](ui.md#fn-shooterHud) | function | ui | — |
| [`og.ui.shopScreen`](ui.md#fn-shopScreen) | function | ui | — |
| [`og.ui.skeleton`](ui.md#fn-skeleton) | function | ui | `og_ui_skeleton` |
| [`og.ui.skeletonGroup`](ui.md#fn-skeletonGroup) | function | ui | — |
| [`og.ui.slider`](ui.md#fn-slider) | function | ui | `og_ui_slider` |
| [`og.ui.space`](ui.md#fn-space) | function | ui | — |
| [`og.ui.spacer`](ui.md#fn-spacer) | function | ui | `og_ui_spacer` |
| [`og.ui.spawnMass`](ui.md#fn-spawnMass) | function | ui | `og_ui_spawn_mass` |
| [`og.ui.speechBubble`](ui.md#fn-speechBubble) | function | ui | — |
| [`og.ui.speedometer`](ui.md#fn-speedometer) | function | ui | — |
| [`og.ui.spinner`](ui.md#fn-spinner) | function | ui | `og_ui_spinner` |
| [`og.ui.splitView`](ui.md#fn-splitView) | function | ui | — |
| [`og.ui.spotlight`](ui.md#fn-spotlight) | function | ui | `og_ui_spotlight` |
| [`og.ui.starRating`](ui.md#fn-starRating) | function | ui | — |
| [`og.ui.statBar`](ui.md#fn-statBar) | function | ui | — |
| [`og.ui.statCompare`](ui.md#fn-statCompare) | function | ui | — |
| [`og.ui.statusGrid`](ui.md#fn-statusGrid) | function | ui | — |
| [`og.ui.stepper`](ui.md#fn-stepper) | function | ui | `og_ui_stepper` |
| [`og.ui.stop`](ui.md#fn-stop) | function | ui | `og_ui_stop` |
| [`og.ui.subtitles`](ui.md#fn-subtitles) | function | ui | — |
| [`og.ui.tabBar`](ui.md#fn-tabBar) | function | ui | — |
| [`og.ui.table`](ui.md#fn-table) | function | ui | — |
| [`og.ui.tabs`](ui.md#fn-tabs) | function | ui | `og_ui_tabs` |
| [`og.ui.telegraph`](ui.md#fn-telegraph) | function | ui | — |
| [`og.ui.text`](ui.md#fn-text) | function | ui | — |
| [`og.ui.textArea`](ui.md#fn-textArea) | function | ui | `og_ui_text_area` |
| [`og.ui.textField`](ui.md#fn-textField) | function | ui | `og_ui_text_field` |
| [`og.ui.theme`](ui.md#fn-theme) | function | ui | `og_ui_theme` |
| [`og.ui.tick`](ui.md#fn-tick) | function | ui | `og_ui_tick` |
| [`og.ui.toast`](ui.md#fn-toast) | function | ui | `og_ui_toast` |
| [`og.ui.toggle`](ui.md#fn-toggle) | function | ui | `og_ui_toggle` |
| [`og.ui.toggleSwitch`](ui.md#fn-toggleSwitch) | function | ui | — |
| [`og.ui.tooltip`](ui.md#fn-tooltip) | function | ui | `og_ui_tooltip` |
| [`og.ui.tooltipCard`](ui.md#fn-tooltipCard) | function | ui | — |
| [`og.ui.transition`](ui.md#fn-transition) | function | ui | — |
| [`og.ui.tree`](ui.md#fn-tree) | function | ui | — |
| [`og.ui.turnOrder`](ui.md#fn-turnOrder) | function | ui | — |
| [`og.ui.tutorialOverlay`](ui.md#fn-tutorialOverlay) | function | ui | — |
| [`og.ui.unitFrame`](ui.md#fn-unitFrame) | function | ui | — |
| [`og.ui.valueSlider`](ui.md#fn-valueSlider) | function | ui | — |
| [`og.ui.viewFrame`](ui.md#fn-viewFrame) | function | ui | — |
| [`og.ui.viewport`](ui.md#fn-viewport) | function | ui | `og_ui_viewport` |
| [`og.ui.visualState`](ui.md#fn-visualState) | function | ui | `og_ui_visual_state` |
| [`og.ui.waypoint`](ui.md#fn-waypoint) | function | ui | — |
| [`og.ui.zoomView`](ui.md#fn-zoomView) | function | ui | `og_ui_zoom_view` |
| [`og.Vehicle`](physics.md#record-Vehicle) | component | physics | — |
| [`og.VehicleWheel`](physics.md#record-VehicleWheel) | component | physics | — |
| [`og.Velocity`](physics.md#record-Velocity) | component | physics | — |
| [`og.VideoPlayer`](video.md#record-VideoPlayer) | component | video | — |
| [`og.Water`](atmosphere.md#record-Water) | component | atmosphere | — |
| [`og.Water2D`](atmosphere.md#record-Water2D) | component | atmosphere | — |
| [`og.WaterLine`](physics.md#record-WaterLine) | component | physics | — |
| [`og.WaterSurface`](physics.md#record-WaterSurface) | component | physics | — |
| [`og.Weather`](atmosphere.md#record-Weather) | resource | atmosphere | — |
| [`og.Wheel`](input.md#record-Wheel) | event | input | — |
| [`og.Wind`](atmosphere.md#record-Wind) | resource | atmosphere | — |
| [`og.WorldRestored`](world.md#record-WorldRestored) | event | world | — |
| [`OrbMeterDesc`](ui.md#type-OrbMeterDesc) | desc struct | ui | — |
| [`Orientation`](ui.md#type-Orientation) | enum | ui | — |
| [`OverheadBar`](ui.md#type-OverheadBar) | desc struct | ui | — |
| [`OverheadBarsDesc`](ui.md#type-OverheadBarsDesc) | desc struct | ui | — |
| [`OverlapDesc`](physics.md#type-OverlapDesc) | desc struct | physics | — |
| [`PageResult`](ui.md#type-PageResult) | struct | ui | — |
| [`PaginationDesc`](ui.md#type-PaginationDesc) | desc struct | ui | — |
| [`Paint`](ui.md#type-Paint) | desc struct | ui | — |
| [`Palette`](render.md#type-Palette) | handle | render | — |
| [`Palette()`](render.md#column-Palette) | column kind | render | — |
| [`PanelDesc`](ui.md#type-PanelDesc) | desc struct | ui | — |
| [`PanelInfo`](ui.md#type-PanelInfo) | struct | ui | — |
| [`PanningModel`](audio.md#type-PanningModel) | enum | audio | — |
| [`PanningModel()`](audio.md#column-PanningModel) | column kind | audio | — |
| [`ParticleEmitter`](render.md#type-ParticleEmitter) | handle | render | — |
| [`ParticleEmitter()`](render.md#column-ParticleEmitter) | column kind | render | — |
| [`PARTICLES_ARMED`](render.md#const-PARTICLES_ARMED) | constant | render | — |
| [`ParticlesDesc`](ui.md#type-ParticlesDesc) | desc struct | ui | — |
| [`ParticleShape`](ui.md#type-ParticleShape) | enum | ui | — |
| [`PartyFramesDesc`](ui.md#type-PartyFramesDesc) | desc struct | ui | — |
| [`PassSize`](probe.md#type-PassSize) | struct | probe | — |
| [`Path`](path.md#type-Path) | handle | path | — |
| [`Path()`](path.md#column-Path) | column kind | path | — |
| [`Path2`](shapes.md#type-Path2) | interface | shapes | — |
| [`Path2.absarc`](shapes.md#member-Path2-absarc) | method | shapes | — |
| [`Path2.absellipse`](shapes.md#member-Path2-absellipse) | method | shapes | — |
| [`Path2.arc`](shapes.md#member-Path2-arc) | method | shapes | — |
| [`Path2.bezierCurveTo`](shapes.md#member-Path2-bezierCurveTo) | method | shapes | — |
| [`Path2.closePath`](shapes.md#member-Path2-closePath) | method | shapes | — |
| [`Path2.ellipse`](shapes.md#member-Path2-ellipse) | method | shapes | — |
| [`Path2.getPoints`](shapes.md#member-Path2-getPoints) | method | shapes | — |
| [`Path2.lineTo`](shapes.md#member-Path2-lineTo) | method | shapes | — |
| [`Path2.moveTo`](shapes.md#member-Path2-moveTo) | method | shapes | — |
| [`Path2.quadraticCurveTo`](shapes.md#member-Path2-quadraticCurveTo) | method | shapes | — |
| [`Path2.setFromPoints`](shapes.md#member-Path2-setFromPoints) | method | shapes | — |
| [`Path2.splineThru`](shapes.md#member-Path2-splineThru) | method | shapes | — |
| [`Path2Class`](shapes.md#type-Path2Class) | interface | shapes | — |
| [`Path2Class.new`](shapes.md#member-Path2Class-new) | method | shapes | — |
| [`PathDesc`](nav.md#type-PathDesc) | desc struct | nav | — |
| [`PathKind`](path.md#type-PathKind) | enum | path | — |
| [`PathKind()`](path.md#column-PathKind) | column kind | path | — |
| [`PathLineDesc`](ui.md#type-PathLineDesc) | desc struct | ui | — |
| [`PathList`](shapes.md#type-PathList) | interface | shapes | — |
| [`PathList.length`](shapes.md#member-PathList-length) | property | shapes | — |
| [`PathList.push`](shapes.md#member-PathList-push) | method | shapes | — |
| [`PathNearest`](path.md#type-PathNearest) | struct | path | — |
| [`PathSample`](path.md#type-PathSample) | struct | path | — |
| [`PauseMenuDesc`](ui.md#type-PauseMenuDesc) | desc struct | ui | — |
| [`PercentSize`](ui.md#type-PercentSize) | struct | ui | — |
| [`PhysicsPhase`](physics.md#type-PhysicsPhase) | struct | physics | — |
| [`PhysicsStats`](physics.md#type-PhysicsStats) | struct | physics | — |
| [`PickedColor`](ui.md#type-PickedColor) | desc struct | ui | — |
| [`PickOperation`](probe.md#type-PickOperation) | struct | probe | — |
| [`PickResult`](probe.md#type-PickResult) | struct | probe | — |
| [`PIN_LEFT`](physics.md#const-PIN_LEFT) | constant | physics | — |
| [`PIN_TOP`](physics.md#const-PIN_TOP) | constant | physics | — |
| [`PIN_TOP_CORNERS`](physics.md#const-PIN_TOP_CORNERS) | constant | physics | — |
| [`PIN_TOP_RINGS`](physics.md#const-PIN_TOP_RINGS) | constant | physics | — |
| [`PinchSpec`](ui.md#type-PinchSpec) | desc struct | ui | — |
| [`PlacementGhostDesc`](ui.md#type-PlacementGhostDesc) | desc struct | ui | — |
| [`Plane`](math.md#type-Plane) | struct | math | — |
| [`plane`](math.md#value-plane) | value | math | — |
| [`PlaneGeometryDesc`](render.md#type-PlaneGeometryDesc) | desc struct | render | — |
| [`PlaneMath`](math.md#type-PlaneMath) | interface | math | — |
| [`PlaneMath.distanceToPoint`](math.md#member-PlaneMath-distanceToPoint) | method | math | — |
| [`PlaneMath.fromCoplanarPoints`](math.md#member-PlaneMath-fromCoplanarPoints) | method | math | — |
| [`PlaneMath.fromNormalAndCoplanarPoint`](math.md#member-PlaneMath-fromNormalAndCoplanarPoint) | method | math | — |
| [`PlaneMath.intersectsBox`](math.md#member-PlaneMath-intersectsBox) | method | math | — |
| [`PlaneMath.intersectsSphere`](math.md#member-PlaneMath-intersectsSphere) | method | math | — |
| [`PlaneMath.normalize`](math.md#member-PlaneMath-normalize) | method | math | — |
| [`PlaneMath.projectPoint`](math.md#member-PlaneMath-projectPoint) | method | math | — |
| [`PlatformerHudDesc`](ui.md#type-PlatformerHudDesc) | desc struct | ui | — |
| [`PlatformError`](platform.md#type-PlatformError) | struct | platform | — |
| [`PlatformInfo`](platform.md#type-PlatformInfo) | struct | platform | — |
| [`PlatformOrder`](platform.md#type-PlatformOrder) | enum | platform | — |
| [`PlatformResult`](platform.md#type-PlatformResult) | struct | platform | — |
| [`PlayDesc`](ui.md#type-PlayDesc) | desc struct | ui | — |
| [`PlayInfo`](ui.md#type-PlayInfo) | struct | ui | — |
| [`PLAYTEST_MAX_TICKS`](probe.md#const-PLAYTEST_MAX_TICKS) | constant | probe | — |
| [`PointDesc`](platform.md#type-PointDesc) | desc struct | platform | — |
| [`PointerAction`](input.md#type-PointerAction) | enum | input | — |
| [`PointerButton`](input.md#type-PointerButton) | enum | input | — |
| [`PointerInput`](input.md#type-PointerInput) | struct | input | — |
| [`PointerLockReport`](input.md#type-PointerLockReport) | struct | input | — |
| [`Pointers.count`](input.md#member-Pointers-count) | method | input | `og_pointers_count` |
| [`Pointers.down`](input.md#member-Pointers-down) | method | input | `og_pointers_down` |
| [`Pointers.position`](input.md#member-Pointers-position) | method | input | `og_pointers_position` |
| [`Pointers.pressed`](input.md#member-Pointers-pressed) | method | input | `og_pointers_pressed` |
| [`Pointers.released`](input.md#member-Pointers-released) | method | input | `og_pointers_released` |
| [`Pointers.start`](input.md#member-Pointers-start) | method | input | `og_pointers_start` |
| [`PointProjection`](physics.md#type-PointProjection) | struct | physics | — |
| [`PointsDesc`](render.md#type-PointsDesc) | desc struct | render | — |
| [`PointShadows`](render.md#type-PointShadows) | enum | render | — |
| [`PoissonDesc`](procgen.md#type-PoissonDesc) | desc struct | procgen | — |
| [`PolyhedronGeometryDesc`](render.md#type-PolyhedronGeometryDesc) | desc struct | render | — |
| [`PopoverDesc`](ui.md#type-PopoverDesc) | desc struct | ui | — |
| [`PopoverResult`](ui.md#type-PopoverResult) | struct | ui | — |
| [`Position`](ui.md#type-Position) | enum | ui | — |
| [`Post`](render.md#type-Post) | handle | render | — |
| [`Post()`](render.md#column-Post) | column kind | render | — |
| [`PostError`](render.md#type-PostError) | struct | render | — |
| [`PowerPreference`](render.md#type-PowerPreference) | enum | render | — |
| [`Presence`](ui.md#type-Presence) | struct | ui | — |
| [`PresenceSpec`](ui.md#type-PresenceSpec) | desc struct | ui | — |
| [`PROBE_GLOBAL`](probe.md#const-PROBE_GLOBAL) | constant | probe | — |
| [`PROBE_PROGRAM_MAX_OPERATIONS`](probe.md#const-PROBE_PROGRAM_MAX_OPERATIONS) | constant | probe | — |
| [`PROBE_VERSION`](probe.md#const-PROBE_VERSION) | constant | probe | — |
| [`PROBE_WIRE_VERSION`](probe.md#const-PROBE_WIRE_VERSION) | constant | probe | — |
| [`ProbeEntityRef`](probe.md#type-ProbeEntityRef) | struct | probe | — |
| [`ProbeHeap`](probe.md#type-ProbeHeap) | struct | probe | — |
| [`ProbeMemory`](probe.md#type-ProbeMemory) | struct | probe | — |
| [`ProbeOperation`](probe.md#type-ProbeOperation) | union | probe | — |
| [`ProbeProgramResult`](probe.md#type-ProbeProgramResult) | struct | probe | — |
| [`ProbeState`](probe.md#type-ProbeState) | struct | probe | — |
| [`ProbeStreamMemory`](probe.md#type-ProbeStreamMemory) | struct | probe | — |
| [`ProbeTiming`](probe.md#type-ProbeTiming) | struct | probe | — |
| [`ProbeTimings`](probe.md#type-ProbeTimings) | struct | probe | — |
| [`ProbeTunable`](probe.md#type-ProbeTunable) | struct | probe | — |
| [`ProbeWorldMemory`](probe.md#type-ProbeWorldMemory) | struct | probe | — |
| [`PROFILE_2D`](physics.md#const-PROFILE_2D) | constant | physics | — |
| [`PROFILE_3D`](physics.md#const-PROFILE_3D) | constant | physics | — |
| [`ProgressBarDesc`](ui.md#type-ProgressBarDesc) | desc struct | ui | — |
| [`ProgressDesc`](ui.md#type-ProgressDesc) | desc struct | ui | — |
| [`ProgressRingDesc`](ui.md#type-ProgressRingDesc) | desc struct | ui | — |
| [`ProgressStyle`](ui.md#type-ProgressStyle) | enum | ui | — |
| [`ProjectPointDesc`](physics.md#type-ProjectPointDesc) | desc struct | physics | — |
| [`PropValue`](ui.md#type-PropValue) | struct | ui | — |
| [`ProseDesc`](ui.md#type-ProseDesc) | desc struct | ui | — |
| [`PushButtonDesc`](ui.md#type-PushButtonDesc) | desc struct | ui | — |
| [`PuzzleHudDesc`](ui.md#type-PuzzleHudDesc) | desc struct | ui | — |
| [`QualityPreset`](render.md#type-QualityPreset) | enum | render | — |
| [`QualityPreset()`](render.md#column-QualityPreset) | column kind | render | — |
| [`quat`](math.md#value-quat) | value | math | — |
| [`Quaternion`](math.md#type-Quaternion) | struct | math | — |
| [`Quaternion()`](reflect.md#column-Quaternion) | column kind | reflect | — |
| [`QuaternionMath`](math.md#type-QuaternionMath) | interface | math | — |
| [`QuaternionMath.fromEuler`](math.md#member-QuaternionMath-fromEuler) | method | math | — |
| [`QuaternionMath.invert`](math.md#member-QuaternionMath-invert) | method | math | — |
| [`QuaternionMath.multiply`](math.md#member-QuaternionMath-multiply) | method | math | — |
| [`QuaternionMath.rotate`](math.md#member-QuaternionMath-rotate) | method | math | — |
| [`QuaternionMath.slerp`](math.md#member-QuaternionMath-slerp) | method | math | — |
| [`Query`](world.md#type-Query) | interface | world | — |
| [`Query.archetypes`](world.md#member-Query-archetypes) | method | world | `og_query_archetypes` |
| [`Query.count`](world.md#member-Query-count) | method | world | `og_query_count` |
| [`QueryDesc`](world.md#type-QueryDesc) | desc struct | world | — |
| [`QueryFilter`](physics.md#type-QueryFilter) | desc struct | physics | — |
| [`RaceSplit`](ui.md#type-RaceSplit) | desc struct | ui | — |
| [`RacingHudDesc`](ui.md#type-RacingHudDesc) | desc struct | ui | — |
| [`RadialDesc`](ui.md#type-RadialDesc) | desc struct | ui | — |
| [`RadialItem`](ui.md#type-RadialItem) | desc struct | ui | — |
| [`RadialMeterDesc`](ui.md#type-RadialMeterDesc) | desc struct | ui | — |
| [`RadialResult`](ui.md#type-RadialResult) | struct | ui | — |
| [`RadioDesc`](ui.md#type-RadioDesc) | desc struct | ui | — |
| [`RadioGroupDesc`](ui.md#type-RadioGroupDesc) | desc struct | ui | — |
| [`RadioOption`](ui.md#type-RadioOption) | desc struct | ui | — |
| [`RadioVariant`](ui.md#type-RadioVariant) | enum | ui | — |
| [`RagdollBoneDesc`](physics.md#type-RagdollBoneDesc) | desc struct | physics | — |
| [`RagdollRigDesc`](physics.md#type-RagdollRigDesc) | desc struct | physics | — |
| [`RaidFramesDesc`](ui.md#type-RaidFramesDesc) | desc struct | ui | — |
| [`RaidMember`](ui.md#type-RaidMember) | desc struct | ui | — |
| [`RaidRole`](ui.md#type-RaidRole) | enum | ui | — |
| [`RangeResult`](ui.md#type-RangeResult) | struct | ui | — |
| [`RangeSliderDesc`](ui.md#type-RangeSliderDesc) | desc struct | ui | — |
| [`Rarity`](ui.md#type-Rarity) | enum | ui | — |
| [`Ray`](math.md#type-Ray) | struct | math | — |
| [`ray`](math.md#value-ray) | value | math | — |
| [`RaycastAllDesc`](physics.md#type-RaycastAllDesc) | desc struct | physics | — |
| [`RaycastDesc`](physics.md#type-RaycastDesc) | desc struct | physics | — |
| [`RaycastGeometryDesc`](render.md#type-RaycastGeometryDesc) | desc struct | render | — |
| [`RaycastHit`](physics.md#type-RaycastHit) | struct | physics | — |
| [`RayMath`](math.md#type-RayMath) | interface | math | — |
| [`RayMath.at`](math.md#member-RayMath-at) | method | math | — |
| [`RayMath.distanceToPoint`](math.md#member-RayMath-distanceToPoint) | method | math | — |
| [`RayMath.intersectBox`](math.md#member-RayMath-intersectBox) | method | math | — |
| [`RayMath.intersectPlane`](math.md#member-RayMath-intersectPlane) | method | math | — |
| [`RayMath.intersectSphere`](math.md#member-RayMath-intersectSphere) | method | math | — |
| [`RayMath.intersectTriangle`](math.md#member-RayMath-intersectTriangle) | method | math | — |
| [`Reaction`](ui.md#type-Reaction) | enum | ui | — |
| [`RebindAnswer`](input.md#type-RebindAnswer) | struct | input | — |
| [`RebindDesc`](input.md#type-RebindDesc) | desc struct | input | — |
| [`RecordFn`](world.md#type-RecordFn) | callback | world | — |
| [`ReducedMotion`](ui.md#type-ReducedMotion) | enum | ui | — |
| [`ReflectionProbeMode`](render.md#type-ReflectionProbeMode) | enum | render | — |
| [`ReflectionProbeMode()`](render.md#column-ReflectionProbeMode) | column kind | render | — |
| [`ReloadReport`](probe.md#type-ReloadReport) | struct | probe | — |
| [`RenderCapabilities`](render.md#type-RenderCapabilities) | struct | render | — |
| [`RenderOptions`](render.md#type-RenderOptions) | desc struct | render | — |
| [`RenderTarget`](render.md#type-RenderTarget) | handle | render | — |
| [`RenderTarget()`](render.md#column-RenderTarget) | column kind | render | — |
| [`RenderTargetDesc`](render.md#type-RenderTargetDesc) | desc struct | render | — |
| [`RenderTargetFormat`](render.md#type-RenderTargetFormat) | enum | render | — |
| [`RenderTargetFormat()`](render.md#column-RenderTargetFormat) | column kind | render | — |
| [`RenderTargetInfo`](render.md#type-RenderTargetInfo) | struct | render | — |
| [`RenderTier`](render.md#type-RenderTier) | enum | render | — |
| [`Repeat`](ui.md#type-Repeat) | enum | ui | — |
| [`REPLAY_SEED_PARAM`](probe.md#const-REPLAY_SEED_PARAM) | constant | probe | — |
| [`ResizeDesc`](tilemap.md#type-ResizeDesc) | desc struct | tilemap | — |
| [`ResourceBarDesc`](ui.md#type-ResourceBarDesc) | desc struct | ui | — |
| [`ResourceItem`](ui.md#type-ResourceItem) | desc struct | ui | — |
| [`Response`](ui.md#type-Response) | struct | ui | — |
| [`RestoreDesc`](world.md#type-RestoreDesc) | desc struct | world | — |
| [`ResultsDesc`](ui.md#type-ResultsDesc) | desc struct | ui | — |
| [`RetargetDesc`](anim.md#type-RetargetDesc) | desc struct | anim | — |
| [`RibbonDesc`](render.md#type-RibbonDesc) | desc struct | render | — |
| [`RibbonPoint`](render.md#type-RibbonPoint) | desc struct | render | — |
| [`RichTextDesc`](ui.md#type-RichTextDesc) | desc struct | ui | — |
| [`RichTextResult`](ui.md#type-RichTextResult) | struct | ui | — |
| [`Rig`](anim.md#type-Rig) | handle | anim | — |
| [`Rig()`](anim.md#column-Rig) | column kind | anim | — |
| [`Rig2D`](anim.md#type-Rig2D) | handle | anim | — |
| [`Rig2D()`](anim.md#column-Rig2D) | column kind | anim | — |
| [`Rig2DClipInfo`](anim.md#type-Rig2DClipInfo) | struct | anim | — |
| [`Rig2DInfo`](anim.md#type-Rig2DInfo) | struct | anim | — |
| [`RingGeometryDesc`](render.md#type-RingGeometryDesc) | desc struct | render | — |
| [`Rng.chance`](rng.md#member-Rng-chance) | method | rng | `og_rng_chance` |
| [`Rng.fork`](rng.md#member-Rng-fork) | method | rng | `og_rng_fork` |
| [`Rng.integer`](rng.md#member-Rng-integer) | method | rng | `og_rng_integer` |
| [`Rng.next`](rng.md#member-Rng-next) | method | rng | `og_rng_next` |
| [`Rng.permutation`](rng.md#member-Rng-permutation) | method | rng | `og_rng_permutation` |
| [`Rng.range`](rng.md#member-Rng-range) | method | rng | `og_rng_range` |
| [`Rng.weighted`](rng.md#member-Rng-weighted) | method | rng | `og_rng_weighted` |
| [`RngStream`](rng.md#type-RngStream) | interface | rng | — |
| [`RngStream.chance`](rng.md#member-RngStream-chance) | method | rng | `og_rng_stream_chance` |
| [`RngStream.fork`](rng.md#member-RngStream-fork) | method | rng | `og_rng_stream_fork` |
| [`RngStream.integer`](rng.md#member-RngStream-integer) | method | rng | `og_rng_stream_integer` |
| [`RngStream.next`](rng.md#member-RngStream-next) | method | rng | `og_rng_stream_next` |
| [`RngStream.permutation`](rng.md#member-RngStream-permutation) | method | rng | `og_rng_stream_permutation` |
| [`RngStream.range`](rng.md#member-RngStream-range) | method | rng | `og_rng_stream_range` |
| [`RngStream.seed`](rng.md#member-RngStream-seed) | property | rng | `og_rng_stream_get_seed` |
| [`RngStream.weighted`](rng.md#member-RngStream-weighted) | method | rng | `og_rng_stream_weighted` |
| [`RoomDesc`](platform.md#type-RoomDesc) | desc struct | platform | — |
| [`RoomInfo`](platform.md#type-RoomInfo) | struct | platform | — |
| [`RoomMessageData`](platform.md#type-RoomMessageData) | struct | platform | — |
| [`RoomPeer`](platform.md#type-RoomPeer) | struct | platform | — |
| [`RootMotion`](anim.md#type-RootMotion) | enum | anim | — |
| [`RootMotion()`](anim.md#column-RootMotion) | column kind | anim | — |
| [`RumbleDesc`](input.md#type-RumbleDesc) | desc struct | input | — |
| [`RunCondition`](system.md#type-RunCondition) | callback | system | — |
| [`RuntimeProbe`](probe.md#type-RuntimeProbe) | interface | probe | — |
| [`RuntimeProbe.accepted`](probe.md#member-RuntimeProbe-accepted) | method | probe | `og_runtime_probe_accepted` |
| [`RuntimeProbe.advance`](probe.md#member-RuntimeProbe-advance) | method | probe | `og_runtime_probe_advance` |
| [`RuntimeProbe.bounds`](probe.md#member-RuntimeProbe-bounds) | method | probe | `og_runtime_probe_bounds` |
| [`RuntimeProbe.entity`](probe.md#member-RuntimeProbe-entity) | method | probe | `og_runtime_probe_entity` |
| [`RuntimeProbe.input`](probe.md#member-RuntimeProbe-input) | method | probe | `og_runtime_probe_input` |
| [`RuntimeProbe.patch`](probe.md#member-RuntimeProbe-patch) | method | probe | `og_runtime_probe_patch` |
| [`RuntimeProbe.pause`](probe.md#member-RuntimeProbe-pause) | method | probe | `og_runtime_probe_pause` |
| [`RuntimeProbe.pick`](probe.md#member-RuntimeProbe-pick) | method | probe | `og_runtime_probe_pick` |
| [`RuntimeProbe.resume`](probe.md#member-RuntimeProbe-resume) | method | probe | `og_runtime_probe_resume` |
| [`RuntimeProbe.run`](probe.md#member-RuntimeProbe-run) | method | probe | `og_runtime_probe_run` |
| [`RuntimeProbe.state`](probe.md#member-RuntimeProbe-state) | method | probe | `og_runtime_probe_state` |
| [`RuntimeProbe.tick`](probe.md#member-RuntimeProbe-tick) | method | probe | `og_runtime_probe_tick` |
| [`RuntimeProbe.version`](probe.md#member-RuntimeProbe-version) | property | probe | `og_runtime_probe_get_version` |
| [`SavedScene`](probe.md#type-SavedScene) | struct | probe | — |
| [`SavedSceneFile`](probe.md#type-SavedSceneFile) | struct | probe | — |
| [`SaveEdit`](world.md#type-SaveEdit) | interface | world | — |
| [`SaveEdit.close`](world.md#member-SaveEdit-close) | method | world | `og_save_edit_close` |
| [`SaveEdit.dropType`](world.md#member-SaveEdit-dropType) | method | world | `og_save_edit_drop_type` |
| [`SaveEdit.has`](world.md#member-SaveEdit-has) | method | world | — |
| [`SaveEdit.records`](world.md#member-SaveEdit-records) | method | world | `og_save_edit_records` |
| [`SaveEdit.renameField`](world.md#member-SaveEdit-renameField) | method | world | `og_save_edit_rename_field` |
| [`SaveEdit.renameType`](world.md#member-SaveEdit-renameType) | method | world | `og_save_edit_rename_type` |
| [`SaveEdit.setRecords`](world.md#member-SaveEdit-setRecords) | method | world | `og_save_edit_set_records` |
| [`SaveEdit.spawn`](world.md#member-SaveEdit-spawn) | method | world | `og_save_edit_spawn` |
| [`SaveEdit.split`](world.md#member-SaveEdit-split) | method | world | — |
| [`SaveEdit.text`](world.md#member-SaveEdit-text) | method | world | `og_save_edit_text` |
| [`SaveEdit.transform`](world.md#member-SaveEdit-transform) | method | world | — |
| [`SaveEdit.types`](world.md#member-SaveEdit-types) | method | world | `og_save_edit_types` |
| [`SaveEdit.version`](world.md#member-SaveEdit-version) | property | world | `og_save_edit_get_version` |
| [`SaveMigration`](world.md#type-SaveMigration) | struct | world | — |
| [`SaveRecord`](world.md#type-SaveRecord) | struct | world | — |
| [`SaveSlotDesc`](platform.md#type-SaveSlotDesc) | desc struct | platform | — |
| [`SaveSpawnDesc`](world.md#type-SaveSpawnDesc) | desc struct | world | — |
| [`SaveType`](world.md#type-SaveType) | struct | world | — |
| [`SceneViewportDesc`](ui.md#type-SceneViewportDesc) | desc struct | ui | — |
| [`SceneViewportInfo`](ui.md#type-SceneViewportInfo) | struct | ui | — |
| [`Scheme`](ui.md#type-Scheme) | enum | ui | — |
| [`ScoreboardDesc`](ui.md#type-ScoreboardDesc) | desc struct | ui | — |
| [`ScorebugDesc`](ui.md#type-ScorebugDesc) | desc struct | ui | — |
| [`ScorebugTeam`](ui.md#type-ScorebugTeam) | desc struct | ui | — |
| [`ScoreColumn`](ui.md#type-ScoreColumn) | desc struct | ui | — |
| [`ScoreFormat`](ui.md#type-ScoreFormat) | enum | ui | — |
| [`ScoreRow`](ui.md#type-ScoreRow) | desc struct | ui | — |
| [`ScoreTeam`](ui.md#type-ScoreTeam) | desc struct | ui | — |
| [`ScreenBlur`](render.md#type-ScreenBlur) | enum | render | — |
| [`ScreenBlur()`](render.md#column-ScreenBlur) | column kind | render | — |
| [`ScreenBox`](probe.md#type-ScreenBox) | struct | probe | — |
| [`ScreenPoint`](probe.md#type-ScreenPoint) | struct | probe | — |
| [`ScreenTransitionDesc`](ui.md#type-ScreenTransitionDesc) | desc struct | ui | — |
| [`ScreenTransitionResult`](ui.md#type-ScreenTransitionResult) | struct | ui | — |
| [`ScreenWipe`](ui.md#type-ScreenWipe) | enum | ui | — |
| [`ScrollAreaDesc`](ui.md#type-ScrollAreaDesc) | desc struct | ui | — |
| [`ScrollAxes`](ui.md#type-ScrollAxes) | enum | ui | — |
| [`ScrollDesc`](ui.md#type-ScrollDesc) | desc struct | ui | — |
| [`SearchBoxDesc`](ui.md#type-SearchBoxDesc) | desc struct | ui | — |
| [`SearchResult`](ui.md#type-SearchResult) | struct | ui | — |
| [`SegmentedDesc`](ui.md#type-SegmentedDesc) | desc struct | ui | — |
| [`SegmentItem`](ui.md#type-SegmentItem) | desc struct | ui | — |
| [`SelectDesc`](ui.md#type-SelectDesc) | desc struct | ui | — |
| [`SelectionRingDesc`](ui.md#type-SelectionRingDesc) | desc struct | ui | — |
| [`SemanticsDesc`](ui.md#type-SemanticsDesc) | desc struct | ui | — |
| [`SensitivityDesc`](input.md#type-SensitivityDesc) | desc struct | input | — |
| [`SeparatorDesc`](ui.md#type-SeparatorDesc) | desc struct | ui | — |
| [`SetRecordsDesc`](world.md#type-SetRecordsDesc) | desc struct | world | — |
| [`SetTileDesc`](tilemap.md#type-SetTileDesc) | desc struct | tilemap | — |
| [`SettingsChanged`](ui.md#type-SettingsChanged) | callback | ui | — |
| [`SettingsDesc`](ui.md#type-SettingsDesc) | desc struct | ui | — |
| [`SettingsResult`](ui.md#type-SettingsResult) | struct | ui | — |
| [`Shader`](render.md#type-Shader) | handle | render | — |
| [`Shader()`](render.md#column-Shader) | column kind | render | — |
| [`ShadowEffect`](ui.md#type-ShadowEffect) | desc struct | ui | — |
| [`ShakeDesc`](ui.md#type-ShakeDesc) | desc struct | ui | — |
| [`Shape`](shapes.md#type-Shape) | interface | shapes | — |
| [`Shape.extractPoints`](shapes.md#member-Shape-extractPoints) | method | shapes | — |
| [`Shape.holes`](shapes.md#member-Shape-holes) | property | shapes | — |
| [`SHAPE_BOX`](physics.md#const-SHAPE_BOX) | constant | physics | — |
| [`SHAPE_CAPSULE`](physics.md#const-SHAPE_CAPSULE) | constant | physics | — |
| [`SHAPE_CONVEX`](physics.md#const-SHAPE_CONVEX) | constant | physics | — |
| [`SHAPE_CYLINDER`](physics.md#const-SHAPE_CYLINDER) | constant | physics | — |
| [`SHAPE_DIAMOND`](physics.md#const-SHAPE_DIAMOND) | constant | physics | — |
| [`SHAPE_HEIGHTFIELD`](physics.md#const-SHAPE_HEIGHTFIELD) | constant | physics | — |
| [`SHAPE_HEXAGON`](physics.md#const-SHAPE_HEXAGON) | constant | physics | — |
| [`SHAPE_SPHERE`](physics.md#const-SHAPE_SPHERE) | constant | physics | — |
| [`SHAPE_TRIMESH`](physics.md#const-SHAPE_TRIMESH) | constant | physics | — |
| [`ShapeBoxDesc`](ui.md#type-ShapeBoxDesc) | desc struct | ui | — |
| [`ShapeCastDesc`](physics.md#type-ShapeCastDesc) | desc struct | physics | — |
| [`ShapeCastHit`](physics.md#type-ShapeCastHit) | struct | physics | — |
| [`ShapeClass`](shapes.md#type-ShapeClass) | interface | shapes | — |
| [`ShapeClass.new`](shapes.md#member-ShapeClass-new) | method | shapes | — |
| [`ShapeDesc`](physics.md#type-ShapeDesc) | desc struct | physics | — |
| [`ShapeGeometryDesc`](render.md#type-ShapeGeometryDesc) | desc struct | render | — |
| [`ShapePoints`](render.md#type-ShapePoints) | struct | render | — |
| [`ShapePreset`](ui.md#type-ShapePreset) | enum | ui | — |
| [`ShapeType`](ui.md#type-ShapeType) | enum | ui | — |
| [`SheetDesc`](ui.md#type-SheetDesc) | desc struct | ui | — |
| [`SheetEdge`](ui.md#type-SheetEdge) | enum | ui | — |
| [`SheetResult`](ui.md#type-SheetResult) | struct | ui | — |
| [`ShooterHudDesc`](ui.md#type-ShooterHudDesc) | desc struct | ui | — |
| [`ShopDesc`](ui.md#type-ShopDesc) | desc struct | ui | — |
| [`ShopItem`](ui.md#type-ShopItem) | desc struct | ui | — |
| [`ShopResult`](ui.md#type-ShopResult) | struct | ui | — |
| [`Side`](render.md#type-Side) | enum | render | — |
| [`Size`](ui.md#type-Size) | union | ui | — |
| [`SizeRange`](ui.md#type-SizeRange) | struct | ui | — |
| [`Skeleton2DBox`](anim.md#type-Skeleton2DBox) | struct | anim | — |
| [`SkeletonAsset`](anim.md#type-SkeletonAsset) | handle | anim | — |
| [`SkeletonAsset()`](anim.md#column-SkeletonAsset) | column kind | anim | — |
| [`SkeletonDesc`](ui.md#type-SkeletonDesc) | desc struct | ui | — |
| [`SkeletonGroupDesc`](ui.md#type-SkeletonGroupDesc) | desc struct | ui | — |
| [`SkeletonShape`](ui.md#type-SkeletonShape) | enum | ui | — |
| [`SliceEdges`](ui.md#type-SliceEdges) | enum | ui | — |
| [`SliderDesc`](ui.md#type-SliderDesc) | desc struct | ui | — |
| [`SliderMark`](ui.md#type-SliderMark) | struct | ui | — |
| [`SliderResult`](ui.md#type-SliderResult) | struct | ui | — |
| [`SlotMove`](ui.md#type-SlotMove) | struct | ui | — |
| [`SnapPoint`](ui.md#type-SnapPoint) | struct | ui | — |
| [`SnapshotDesc`](world.md#type-SnapshotDesc) | desc struct | world | — |
| [`SnapshotInfo`](probe.md#type-SnapshotInfo) | struct | probe | — |
| [`SOFT_BOX`](physics.md#const-SOFT_BOX) | constant | physics | — |
| [`SOFT_MESH`](physics.md#const-SOFT_MESH) | constant | physics | — |
| [`SOFT_SPHERE`](physics.md#const-SOFT_SPHERE) | constant | physics | — |
| [`SoftBodyPoints`](physics.md#type-SoftBodyPoints) | struct | physics | — |
| [`SolidFill`](ui.md#type-SolidFill) | desc struct | ui | — |
| [`SolidGeometryDesc`](render.md#type-SolidGeometryDesc) | desc struct | render | — |
| [`Sound`](load.md#type-Sound) | handle | load | — |
| [`Sound()`](load.md#column-Sound) | column kind | load | — |
| [`SOUND_CULLED`](audio.md#const-SOUND_CULLED) | constant | audio | — |
| [`SOUND_FINISHED`](audio.md#const-SOUND_FINISHED) | constant | audio | — |
| [`SOUND_REMOVED`](audio.md#const-SOUND_REMOVED) | constant | audio | — |
| [`SOUND_REPLACED`](audio.md#const-SOUND_REPLACED) | constant | audio | — |
| [`SOUND_RETRIGGERED`](audio.md#const-SOUND_RETRIGGERED) | constant | audio | — |
| [`SOUND_STOPPED`](audio.md#const-SOUND_STOPPED) | constant | audio | — |
| [`SoundDesc`](audio.md#type-SoundDesc) | desc struct | audio | — |
| [`SoundEvent`](ui.md#type-SoundEvent) | struct | ui | — |
| [`SoundInfo`](load.md#type-SoundInfo) | struct | load | — |
| [`SpaceDesc`](ui.md#type-SpaceDesc) | desc struct | ui | — |
| [`SpaceStep`](ui.md#type-SpaceStep) | enum | ui | — |
| [`SpatialDesc`](world.md#type-SpatialDesc) | desc struct | world | — |
| [`SpatialIndex`](world.md#type-SpatialIndex) | interface | world | — |
| [`SpatialIndex.count`](world.md#member-SpatialIndex-count) | property | world | `og_spatial_index_get_count` |
| [`SpatialIndex.nearest`](world.md#member-SpatialIndex-nearest) | method | world | `og_spatial_index_nearest` |
| [`SpatialIndex.within`](world.md#member-SpatialIndex-within) | method | world | `og_spatial_index_within` |
| [`SpawnDesc`](world.md#type-SpawnDesc) | desc struct | world | — |
| [`SpeechBubbleDesc`](ui.md#type-SpeechBubbleDesc) | desc struct | ui | — |
| [`SpeedometerDesc`](ui.md#type-SpeedometerDesc) | desc struct | ui | — |
| [`SpeedoStyle`](ui.md#type-SpeedoStyle) | enum | ui | — |
| [`Sphere`](math.md#type-Sphere) | struct | math | — |
| [`sphere`](math.md#value-sphere) | value | math | — |
| [`SphereGeometryDesc`](render.md#type-SphereGeometryDesc) | desc struct | render | — |
| [`SphereMath`](math.md#type-SphereMath) | interface | math | — |
| [`SphereMath.applyMatrix4`](math.md#member-SphereMath-applyMatrix4) | method | math | — |
| [`SphereMath.containsPoint`](math.md#member-SphereMath-containsPoint) | method | math | — |
| [`SphereMath.distanceToPoint`](math.md#member-SphereMath-distanceToPoint) | method | math | — |
| [`SphereMath.fromPoints`](math.md#member-SphereMath-fromPoints) | method | math | — |
| [`SphereMath.getBoundingBox`](math.md#member-SphereMath-getBoundingBox) | method | math | — |
| [`SphereMath.intersectsBox`](math.md#member-SphereMath-intersectsBox) | method | math | — |
| [`SphereMath.intersectsSphere`](math.md#member-SphereMath-intersectsSphere) | method | math | — |
| [`SplitFn`](world.md#type-SplitFn) | callback | world | — |
| [`SplitResult`](ui.md#type-SplitResult) | struct | ui | — |
| [`SplitViewDesc`](ui.md#type-SplitViewDesc) | desc struct | ui | — |
| [`SpotlightDesc`](ui.md#type-SpotlightDesc) | desc struct | ui | — |
| [`SpringColliderShape`](anim.md#type-SpringColliderShape) | enum | anim | — |
| [`SpringColliderShape()`](anim.md#column-SpringColliderShape) | column kind | anim | — |
| [`SpringSpace`](anim.md#type-SpringSpace) | enum | anim | — |
| [`SpringSpace()`](anim.md#column-SpringSpace) | column kind | anim | — |
| [`SpringSpec`](ui.md#type-SpringSpec) | desc struct | ui | — |
| [`SpriteBlend`](render.md#type-SpriteBlend) | enum | render | — |
| [`SpriteBlend()`](render.md#column-SpriteBlend) | column kind | render | — |
| [`SpriteDesc`](render.md#type-SpriteDesc) | desc struct | render | — |
| [`SpriteFacing`](render.md#type-SpriteFacing) | enum | render | — |
| [`SpriteFacing()`](render.md#column-SpriteFacing) | column kind | render | — |
| [`SquashDesc`](tween.md#type-SquashDesc) | desc struct | tween | — |
| [`StandardMaterialDesc`](render.md#type-StandardMaterialDesc) | desc struct | render | — |
| [`StarRatingDesc`](ui.md#type-StarRatingDesc) | desc struct | ui | — |
| [`StartFn`](game.md#type-StartFn) | callback | game | — |
| [`StatBarDesc`](ui.md#type-StatBarDesc) | desc struct | ui | — |
| [`StatBarStyle`](ui.md#type-StatBarStyle) | enum | ui | — |
| [`StatCompareDesc`](ui.md#type-StatCompareDesc) | desc struct | ui | — |
| [`StatCompareRow`](ui.md#type-StatCompareRow) | desc struct | ui | — |
| [`StateDesc`](system.md#type-StateDesc) | struct | system | — |
| [`StateEvent`](ui.md#type-StateEvent) | struct | ui | — |
| [`StateGroup`](ui.md#type-StateGroup) | struct | ui | — |
| [`StateTransition`](ui.md#type-StateTransition) | struct | ui | — |
| [`StateValue`](ui.md#type-StateValue) | struct | ui | — |
| [`StateWhen`](system.md#type-StateWhen) | enum | system | — |
| [`StatRow`](ui.md#type-StatRow) | desc struct | ui | — |
| [`StatusEffect`](ui.md#type-StatusEffect) | desc struct | ui | — |
| [`StatusGridDesc`](ui.md#type-StatusGridDesc) | desc struct | ui | — |
| [`StatusGridResult`](ui.md#type-StatusGridResult) | struct | ui | — |
| [`StepperDesc`](ui.md#type-StepperDesc) | desc struct | ui | — |
| [`StepperResult`](ui.md#type-StepperResult) | struct | ui | — |
| [`StoreSetDesc`](platform.md#type-StoreSetDesc) | desc struct | platform | — |
| [`str()`](reflect.md#column-str) | column kind | reflect | — |
| [`STREAM_FAILED`](stream.md#const-STREAM_FAILED) | constant | stream | — |
| [`STREAM_LOADING`](stream.md#const-STREAM_LOADING) | constant | stream | — |
| [`STREAM_QUEUED`](stream.md#const-STREAM_QUEUED) | constant | stream | — |
| [`STREAM_RESIDENT`](stream.md#const-STREAM_RESIDENT) | constant | stream | — |
| [`STREAM_UNLOADED`](stream.md#const-STREAM_UNLOADED) | constant | stream | — |
| [`StreamBudget`](stream.md#type-StreamBudget) | struct | stream | — |
| [`StreamState`](stream.md#type-StreamState) | enum | stream | — |
| [`StreamStatus`](stream.md#type-StreamStatus) | struct | stream | — |
| [`StripBlend`](render.md#type-StripBlend) | enum | render | — |
| [`StripBlend()`](render.md#column-StripBlend) | column kind | render | — |
| [`Stroke`](ui.md#type-Stroke) | desc struct | ui | — |
| [`StrokeAlign`](ui.md#type-StrokeAlign) | enum | ui | — |
| [`Style`](load.md#type-Style) | handle | load | — |
| [`Style()`](load.md#column-Style) | column kind | load | — |
| [`StylePart`](ui.md#type-StylePart) | enum | ui | — |
| [`StyliseSpace`](render.md#type-StyliseSpace) | enum | render | — |
| [`StyliseSpace()`](render.md#column-StyliseSpace) | column kind | render | — |
| [`SubtitleLine`](ui.md#type-SubtitleLine) | desc struct | ui | — |
| [`SubtitlesDesc`](ui.md#type-SubtitlesDesc) | desc struct | ui | — |
| [`SubtitleSettings`](ui.md#type-SubtitleSettings) | desc struct | ui | — |
| [`SubtitleSize`](ui.md#type-SubtitleSize) | enum | ui | — |
| [`SweepDesc`](probe.md#type-SweepDesc) | desc struct | probe | — |
| [`SweepFrame`](probe.md#type-SweepFrame) | struct | probe | — |
| [`SweepGeometryDesc`](render.md#type-SweepGeometryDesc) | desc struct | render | — |
| [`SweepKey`](render.md#type-SweepKey) | struct | render | — |
| [`SweepResult`](probe.md#type-SweepResult) | struct | probe | — |
| [`SwitchResult`](ui.md#type-SwitchResult) | struct | ui | — |
| [`System`](system.md#type-System) | interface | system | — |
| [`System.name`](system.md#member-System-name) | property | system | `og_system_get_name` |
| [`System.reads`](system.md#member-System-reads) | property | system | `og_system_get_reads` |
| [`System.writes`](system.md#member-System-writes) | property | system | `og_system_get_writes` |
| [`SystemDesc`](system.md#type-SystemDesc) | desc struct | system | — |
| [`SystemDump`](probe.md#type-SystemDump) | struct | probe | — |
| [`SystemRun`](system.md#type-SystemRun) | callback | system | — |
| [`TaaHistory`](render.md#type-TaaHistory) | enum | render | — |
| [`TabBarDesc`](ui.md#type-TabBarDesc) | desc struct | ui | — |
| [`TabBarResult`](ui.md#type-TabBarResult) | struct | ui | — |
| [`TabItem`](ui.md#type-TabItem) | desc struct | ui | — |
| [`TableColumn`](ui.md#type-TableColumn) | desc struct | ui | — |
| [`TableDesc`](ui.md#type-TableDesc) | desc struct | ui | — |
| [`TableResult`](ui.md#type-TableResult) | struct | ui | — |
| [`TabsDesc`](ui.md#type-TabsDesc) | desc struct | ui | — |
| [`TabVariant`](ui.md#type-TabVariant) | enum | ui | — |
| [`TelegraphDesc`](ui.md#type-TelegraphDesc) | desc struct | ui | — |
| [`TelegraphShape`](ui.md#type-TelegraphShape) | enum | ui | — |
| [`TerrainBounds`](terrain.md#type-TerrainBounds) | struct | terrain | — |
| [`TerrainBrush`](terrain.md#type-TerrainBrush) | desc struct | terrain | — |
| [`TerrainBrushMode`](terrain.md#type-TerrainBrushMode) | enum | terrain | — |
| [`TerrainData`](terrain.md#type-TerrainData) | handle | terrain | — |
| [`TerrainData()`](terrain.md#column-TerrainData) | column kind | terrain | — |
| [`TerrainEdit`](terrain.md#type-TerrainEdit) | struct | terrain | — |
| [`TerrainInfo`](terrain.md#type-TerrainInfo) | struct | terrain | — |
| [`TerrainLayerDesc`](terrain.md#type-TerrainLayerDesc) | desc struct | terrain | — |
| [`TerrainSample`](terrain.md#type-TerrainSample) | struct | terrain | — |
| [`TerrainSampleDesc`](terrain.md#type-TerrainSampleDesc) | desc struct | terrain | — |
| [`TextAlign`](ui.md#type-TextAlign) | enum | ui | — |
| [`TextAreaDesc`](ui.md#type-TextAreaDesc) | desc struct | ui | — |
| [`TextBlockDesc`](ui.md#type-TextBlockDesc) | desc struct | ui | — |
| [`TextDesc`](render.md#type-TextDesc) | desc struct | render | — |
| [`TextEffectsDesc`](ui.md#type-TextEffectsDesc) | desc struct | ui | — |
| [`TextEntry`](input.md#type-TextEntry) | struct | input | — |
| [`TextMeasureDesc`](render.md#type-TextMeasureDesc) | desc struct | render | — |
| [`TextMetrics`](render.md#type-TextMetrics) | struct | render | — |
| [`TextOutlineDesc`](ui.md#type-TextOutlineDesc) | desc struct | ui | — |
| [`TextRole`](ui.md#type-TextRole) | enum | ui | — |
| [`TextShadowDesc`](ui.md#type-TextShadowDesc) | desc struct | ui | — |
| [`TextTone`](ui.md#type-TextTone) | enum | ui | — |
| [`Texture`](render.md#type-Texture) | handle | render | — |
| [`Texture()`](render.md#column-Texture) | column kind | render | — |
| [`TextureArray`](load.md#type-TextureArray) | handle | load | — |
| [`TextureArray()`](load.md#column-TextureArray) | column kind | load | — |
| [`TextureArrayInfo`](load.md#type-TextureArrayInfo) | struct | load | — |
| [`ThemeColor`](ui.md#type-ThemeColor) | enum | ui | — |
| [`TileCell`](tilemap.md#type-TileCell) | struct | tilemap | — |
| [`TileLayerDesc`](tilemap.md#type-TileLayerDesc) | desc struct | tilemap | — |
| [`TileLayerInfo`](tilemap.md#type-TileLayerInfo) | struct | tilemap | — |
| [`TilemapAsset`](tilemap.md#type-TilemapAsset) | handle | tilemap | — |
| [`TilemapAsset()`](tilemap.md#column-TilemapAsset) | column kind | tilemap | — |
| [`TilemapDesc`](tilemap.md#type-TilemapDesc) | desc struct | tilemap | — |
| [`TilemapGridDesc`](nav.md#type-TilemapGridDesc) | desc struct | nav | — |
| [`TilemapInfo`](tilemap.md#type-TilemapInfo) | struct | tilemap | — |
| [`TileObject`](tilemap.md#type-TileObject) | struct | tilemap | — |
| [`TileOrientation`](tilemap.md#type-TileOrientation) | enum | tilemap | — |
| [`TimelineAction`](ui.md#type-TimelineAction) | union | ui | — |
| [`TimelineAsset`](timeline.md#type-TimelineAsset) | handle | timeline | — |
| [`TimelineAsset()`](timeline.md#column-TimelineAsset) | column kind | timeline | — |
| [`TimelineEvent`](ui.md#type-TimelineEvent) | struct | ui | — |
| [`TimelineInfo`](timeline.md#type-TimelineInfo) | struct | timeline | — |
| [`TimelineKey`](ui.md#type-TimelineKey) | struct | ui | — |
| [`TimelineLabel`](ui.md#type-TimelineLabel) | struct | ui | — |
| [`TimelineTrack`](ui.md#type-TimelineTrack) | struct | ui | — |
| [`TimelineTrackInfo`](timeline.md#type-TimelineTrackInfo) | struct | timeline | — |
| [`TimerMode`](ui.md#type-TimerMode) | enum | ui | — |
| [`ToastDesc`](ui.md#type-ToastDesc) | desc struct | ui | — |
| [`ToggleResult`](ui.md#type-ToggleResult) | struct | ui | — |
| [`ToggleSwitchDesc`](ui.md#type-ToggleSwitchDesc) | desc struct | ui | — |
| [`TokenRef`](ui.md#type-TokenRef) | struct | ui | — |
| [`Tone`](ui.md#type-Tone) | enum | ui | — |
| [`ToneMapping`](render.md#type-ToneMapping) | enum | render | — |
| [`ToneMapping()`](render.md#column-ToneMapping) | column kind | render | — |
| [`TooltipCardDesc`](ui.md#type-TooltipCardDesc) | desc struct | ui | — |
| [`TooltipCardResult`](ui.md#type-TooltipCardResult) | struct | ui | — |
| [`TooltipLink`](ui.md#type-TooltipLink) | desc struct | ui | — |
| [`TooltipStat`](ui.md#type-TooltipStat) | desc struct | ui | — |
| [`TorusGeometryDesc`](render.md#type-TorusGeometryDesc) | desc struct | render | — |
| [`TouchZone`](input.md#type-TouchZone) | desc struct | input | — |
| [`Track`](ui.md#type-Track) | union | ui | — |
| [`TrackAuto`](ui.md#type-TrackAuto) | struct | ui | — |
| [`TrackFr`](ui.md#type-TrackFr) | struct | ui | — |
| [`TrackPercent`](ui.md#type-TrackPercent) | struct | ui | — |
| [`TrackPx`](ui.md#type-TrackPx) | struct | ui | — |
| [`TrackRange`](ui.md#type-TrackRange) | struct | ui | — |
| [`TransformDesc`](world.md#type-TransformDesc) | desc struct | world | — |
| [`Transition`](ui.md#type-Transition) | union | ui | — |
| [`TransitionDesc`](ui.md#type-TransitionDesc) | desc struct | ui | — |
| [`TransitionKind`](ui.md#type-TransitionKind) | enum | ui | — |
| [`TreeDesc`](ui.md#type-TreeDesc) | desc struct | ui | — |
| [`TreeItem`](ui.md#type-TreeItem) | desc struct | ui | — |
| [`TreeResult`](ui.md#type-TreeResult) | struct | ui | — |
| [`TubeGeometryDesc`](render.md#type-TubeGeometryDesc) | desc struct | render | — |
| [`TurnOrderDesc`](ui.md#type-TurnOrderDesc) | desc struct | ui | — |
| [`TurnUnit`](ui.md#type-TurnUnit) | desc struct | ui | — |
| [`TutorialDesc`](ui.md#type-TutorialDesc) | desc struct | ui | — |
| [`TutorialResult`](ui.md#type-TutorialResult) | struct | ui | — |
| [`TutorialStep`](ui.md#type-TutorialStep) | desc struct | ui | — |
| [`Tween`](tween.md#type-Tween) | handle | tween | — |
| [`Tween()`](tween.md#column-Tween) | column kind | tween | — |
| [`TweenDesc`](tween.md#type-TweenDesc) | desc struct | tween | — |
| [`TweenGroupDesc`](tween.md#type-TweenGroupDesc) | desc struct | tween | — |
| [`TweenSpec`](ui.md#type-TweenSpec) | desc struct | ui | — |
| [`TypeStep`](ui.md#type-TypeStep) | struct | ui | — |
| [`TypeStepName`](ui.md#type-TypeStepName) | enum | ui | — |
| [`u32()`](reflect.md#column-u32) | column kind | reflect | — |
| [`UI_LAYER`](render.md#const-UI_LAYER) | constant | render | — |
| [`UiAction`](ui.md#type-UiAction) | callback | ui | — |
| [`UiActionDesc`](ui.md#type-UiActionDesc) | struct | ui | — |
| [`UiActionHint`](ui.md#type-UiActionHint) | struct | ui | — |
| [`UiActionTrigger`](ui.md#type-UiActionTrigger) | enum | ui | — |
| [`UiBuild`](ui.md#type-UiBuild) | callback | ui | — |
| [`UiDesc`](ui.md#type-UiDesc) | desc struct | ui | — |
| [`UiDevice`](ui.md#type-UiDevice) | enum | ui | — |
| [`UiDocEvent`](ui.md#type-UiDocEvent) | struct | ui | — |
| [`UiDocResponse`](ui.md#type-UiDocResponse) | struct | ui | — |
| [`UiDump`](probe.md#type-UiDump) | struct | probe | — |
| [`UiElement`](probe.md#type-UiElement) | struct | probe | — |
| [`UiGlyph`](ui.md#type-UiGlyph) | struct | ui | — |
| [`UiItemBuild`](ui.md#type-UiItemBuild) | callback | ui | — |
| [`UiLayer`](ui.md#type-UiLayer) | struct | ui | — |
| [`UiLayoutStats`](ui.md#type-UiLayoutStats) | struct | ui | — |
| [`UiMaterial`](ui.md#type-UiMaterial) | handle | ui | — |
| [`UiMaterial()`](ui.md#column-UiMaterial) | column kind | ui | — |
| [`UiMotionEvent`](ui.md#type-UiMotionEvent) | struct | ui | — |
| [`UiOverlap`](ui.md#type-UiOverlap) | struct | ui | — |
| [`UiPatch`](ui.md#type-UiPatch) | desc struct | ui | — |
| [`UiPointer`](ui.md#type-UiPointer) | struct | ui | — |
| [`UiProp`](ui.md#type-UiProp) | enum | ui | — |
| [`UiQuery`](ui.md#type-UiQuery) | desc struct | ui | — |
| [`UiRect`](ui.md#type-UiRect) | struct | ui | — |
| [`UiRibbonDesc`](ui.md#type-UiRibbonDesc) | desc struct | ui | — |
| [`UiSounds`](ui.md#type-UiSounds) | desc struct | ui | — |
| [`UiSurface`](probe.md#type-UiSurface) | struct | probe | — |
| [`UiTheme`](ui.md#type-UiTheme) | struct | ui | — |
| [`UiTimeline`](ui.md#type-UiTimeline) | desc struct | ui | — |
| [`UiTransform`](ui.md#type-UiTransform) | desc struct | ui | — |
| [`UiVariant`](ui.md#type-UiVariant) | struct | ui | — |
| [`UnitClass`](ui.md#type-UnitClass) | enum | ui | — |
| [`UnitFrameDesc`](ui.md#type-UnitFrameDesc) | desc struct | ui | — |
| [`UnitFrameSize`](ui.md#type-UnitFrameSize) | enum | ui | — |
| [`ValueFormat`](ui.md#type-ValueFormat) | desc struct | ui | — |
| [`ValueSliderDesc`](ui.md#type-ValueSliderDesc) | desc struct | ui | — |
| [`ValueSliderResult`](ui.md#type-ValueSliderResult) | struct | ui | — |
| [`vec2`](math.md#value-vec2) | value | math | — |
| [`vec3`](math.md#value-vec3) | value | math | — |
| [`Vector2`](math.md#type-Vector2) | struct | math | — |
| [`Vector2()`](reflect.md#column-Vector2) | column kind | reflect | — |
| [`Vector2Math`](math.md#type-Vector2Math) | interface | math | — |
| [`Vector2Math.add`](math.md#member-Vector2Math-add) | method | math | — |
| [`Vector2Math.cross`](math.md#member-Vector2Math-cross) | method | math | — |
| [`Vector2Math.distance`](math.md#member-Vector2Math-distance) | method | math | — |
| [`Vector2Math.dot`](math.md#member-Vector2Math-dot) | method | math | — |
| [`Vector2Math.length`](math.md#member-Vector2Math-length) | method | math | — |
| [`Vector2Math.lerp`](math.md#member-Vector2Math-lerp) | method | math | — |
| [`Vector2Math.normalize`](math.md#member-Vector2Math-normalize) | method | math | — |
| [`Vector2Math.rotateAround`](math.md#member-Vector2Math-rotateAround) | method | math | — |
| [`Vector2Math.scale`](math.md#member-Vector2Math-scale) | method | math | — |
| [`Vector2Math.sub`](math.md#member-Vector2Math-sub) | method | math | — |
| [`Vector3`](math.md#type-Vector3) | struct | math | — |
| [`Vector3()`](reflect.md#column-Vector3) | column kind | reflect | — |
| [`Vector3Math`](math.md#type-Vector3Math) | interface | math | — |
| [`Vector3Math.add`](math.md#member-Vector3Math-add) | method | math | — |
| [`Vector3Math.applyQuaternion`](math.md#member-Vector3Math-applyQuaternion) | method | math | — |
| [`Vector3Math.cross`](math.md#member-Vector3Math-cross) | method | math | — |
| [`Vector3Math.distance`](math.md#member-Vector3Math-distance) | method | math | — |
| [`Vector3Math.dot`](math.md#member-Vector3Math-dot) | method | math | — |
| [`Vector3Math.length`](math.md#member-Vector3Math-length) | method | math | — |
| [`Vector3Math.lerp`](math.md#member-Vector3Math-lerp) | method | math | — |
| [`Vector3Math.normalize`](math.md#member-Vector3Math-normalize) | method | math | — |
| [`Vector3Math.scale`](math.md#member-Vector3Math-scale) | method | math | — |
| [`Vector3Math.sub`](math.md#member-Vector3Math-sub) | method | math | — |
| [`VectorArt`](ui.md#type-VectorArt) | handle | ui | — |
| [`VectorArt()`](ui.md#column-VectorArt) | column kind | ui | — |
| [`VectorFill`](ui.md#type-VectorFill) | desc struct | ui | — |
| [`VEHICLE_ARCADE`](physics.md#const-VEHICLE_ARCADE) | constant | physics | — |
| [`VEHICLE_DRIVETRAIN`](physics.md#const-VEHICLE_DRIVETRAIN) | constant | physics | — |
| [`Video`](video.md#type-Video) | handle | video | — |
| [`Video()`](video.md#column-Video) | column kind | video | — |
| [`VideoInfo`](video.md#type-VideoInfo) | struct | video | — |
| [`ViewFrameDesc`](ui.md#type-ViewFrameDesc) | desc struct | ui | — |
| [`ViewFrameResult`](ui.md#type-ViewFrameResult) | struct | ui | — |
| [`Viewport`](ui.md#type-Viewport) | struct | ui | — |
| [`VisualState`](ui.md#type-VisualState) | struct | ui | — |
| [`VolumetricFog`](render.md#type-VolumetricFog) | enum | render | — |
| [`WaypointDesc`](ui.md#type-WaypointDesc) | desc struct | ui | — |
| [`WaypointKind`](ui.md#type-WaypointKind) | enum | ui | — |
| [`WEATHER_DUST`](atmosphere.md#const-WEATHER_DUST) | constant | atmosphere | — |
| [`WEATHER_MIST`](atmosphere.md#const-WEATHER_MIST) | constant | atmosphere | — |
| [`WEATHER_NONE`](atmosphere.md#const-WEATHER_NONE) | constant | atmosphere | — |
| [`WEATHER_RAIN`](atmosphere.md#const-WEATHER_RAIN) | constant | atmosphere | — |
| [`WEATHER_SNOW`](atmosphere.md#const-WEATHER_SNOW) | constant | atmosphere | — |
| [`WfcCell`](procgen.md#type-WfcCell) | struct | procgen | — |
| [`WfcDesc`](procgen.md#type-WfcDesc) | desc struct | procgen | — |
| [`WfcResult`](procgen.md#type-WfcResult) | struct | procgen | — |
| [`WfcTile`](procgen.md#type-WfcTile) | struct | procgen | — |
| [`WheelInput`](input.md#type-WheelInput) | struct | input | — |
| [`WheelPose`](physics.md#type-WheelPose) | struct | physics | — |
| [`WipeSide`](ui.md#type-WipeSide) | enum | ui | — |
| [`World`](world.md#type-World) | interface | world | — |
| [`World.alive`](world.md#member-World-alive) | method | world | `og_world_alive` |
| [`World.children`](world.md#member-World-children) | method | world | `og_world_children` |
| [`World.despawn`](world.md#member-World-despawn) | method | world | `og_world_despawn` |
| [`World.emit`](world.md#member-World-emit) | method | world | `og_world_emit` |
| [`World.events`](world.md#member-World-events) | method | world | `og_world_events` |
| [`World.find`](world.md#member-World-find) | method | world | `og_world_find` |
| [`World.get`](world.md#member-World-get) | method | world | `og_world_get` |
| [`World.has`](world.md#member-World-has) | method | world | `og_world_has` |
| [`World.insert`](world.md#member-World-insert) | method | world | `og_world_insert` |
| [`World.instantiate`](world.md#member-World-instantiate) | method | world | `og_world_instantiate` |
| [`World.nameOf`](world.md#member-World-nameOf) | method | world | `og_world_name_of` |
| [`World.openSave`](world.md#member-World-openSave) | method | world | `og_world_open_save` |
| [`World.prefabOf`](world.md#member-World-prefabOf) | method | world | `og_world_prefab_of` |
| [`World.query`](world.md#member-World-query) | method | world | `og_world_query` |
| [`World.remove`](world.md#member-World-remove) | method | world | `og_world_remove` |
| [`World.removed`](world.md#member-World-removed) | method | world | `og_world_removed` |
| [`World.resource`](world.md#member-World-resource) | method | world | `og_world_resource` |
| [`World.restore`](world.md#member-World-restore) | method | world | `og_world_restore` |
| [`World.set`](world.md#member-World-set) | method | world | `og_world_set` |
| [`World.setState`](world.md#member-World-setState) | method | world | `og_world_set_state` |
| [`World.snapshot`](world.md#member-World-snapshot) | method | world | `og_world_snapshot` |
| [`World.spatial`](world.md#member-World-spatial) | method | world | `og_world_spatial` |
| [`World.spawn`](world.md#member-World-spawn) | method | world | `og_world_spawn` |
| [`World.state`](world.md#member-World-state) | method | world | `og_world_state` |
| [`World.tick`](world.md#member-World-tick) | property | world | `og_world_get_tick` |
| [`WorldAnchorDesc`](ui.md#type-WorldAnchorDesc) | desc struct | ui | — |
| [`WorldDump`](probe.md#type-WorldDump) | struct | probe | — |
| [`WorldOcclusion`](ui.md#type-WorldOcclusion) | enum | ui | — |
| [`WorldPanelDesc`](ui.md#type-WorldPanelDesc) | desc struct | ui | — |
| [`WorldTarget`](ui.md#type-WorldTarget) | desc struct | ui | — |
| [`ZoomResult`](ui.md#type-ZoomResult) | struct | ui | — |
| [`ZoomViewDesc`](ui.md#type-ZoomViewDesc) | desc struct | ui | — |
