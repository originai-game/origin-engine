<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The Origin Engine API

`@origingame/engine`, API version 1: 52 modules, generated from `crates/og-api/api.ogt`. Each module has an overview page and every function, component and type a page of its own under `<module>/`; every signature is quoted from the IDL. `og api find <words>` ranks every entry; `og api show <name>` prints one.

The Origin Engine API. Game code declares components and systems, and reads the world through typed views over engine memory.

Every failure is a returned Result; desc objects may omit any field; nothing reads the wall clock inside a tick.

## Modules

| Module | TypeScript | Publish builds | Summary |
|---|---|---|---|
| [`math`](math.md) | types and tokens | kept | Plain math values in the three.js vocabulary. They are data: the engine owns the operations, so results are identical on every host. |
| [`reflect`](reflect.md) | named exports | kept | Reflection: components and resources are declared once with their field layout and metadata, and registered by name. |
| [`world`](world.md) | types and tokens | kept | The world: an archetype ECS. Entities are generational handles; components are columns in engine memory. |
| [`system`](system.md) | named exports | kept | Systems: functions the engine calls once per tick with their matched archetypes. |
| [`game`](game.md) | `og.*` | kept | The game entry: what the bundle's default export declares. |
| [`transform`](transform.md) | `og.transform.*` | kept | Transforms in the three.js vocabulary: position, quaternion and scale, composed into matrixWorld down the parent chain. |
| [`time`](time.md) | `og.time.*` | kept | Simulation time. Game code never reads the wall clock; it reads Time. |
| [`rng`](rng.md) | types and tokens | kept | Deterministic randomness. Every random draw comes from an Rng stream; the sequence for a seed is identical on every host and in the Web scaffold's rng.ts. |
| [`procgen`](procgen.md) | `og.procgen.*` | kept | Procedural generation: deterministic noise and FBM, Poisson-disk sampling, BSP dungeons, wave function collapse, eroded heightmaps, plants and Voronoi mesh fracture. Bulk buffers are TypedArrays. |
| [`input`](input.md) | `og.input.*` | kept | Input as the game sees it: the Input resource for polling, events for edges, og.Pointers for several pointers at once (multi-touch) and og.Gamepad for up to eight pads. All of it is world state, so it replays. |
| [`camera`](camera.md) | `og.camera.*` | kept | The main camera, as a resource with three.js PerspectiveCamera and OrthographicCamera parameter names. |
| [`render`](render.md) | `og.render.*` | kept | What is drawn: meshes with materials. Draws are instanced by (geometry, material) automatically; there is no per-object draw call. |
| [`shapes`](shapes.md) | types and tokens | kept | Two-dimensional outlines in three.js's words: og.Shape and og.Path2 are three.js's Shape and Path, built by the same calls (moveTo, lineTo, bezierCurveTo, arc, absarc, ellipse, closePath, ...) and flattened to points by getPoints. og.render.shapeGeometry, extrudeGeometry and sweepGeometry take them; only their points cross to the engine. |
| [`load`](load.md) | `og.load.*` | kept | Load cooked output: `og cook` turns content/**/*.ts and imported assets into cooked/ (.ogb blobs and the internal manifest cooked/.cook.ogt). The runtime reads only cooked/. |
| [`physics`](physics.md) | `og.physics.*` | kept | Rigid bodies over rapier (rapier3d in the 3D profile, rapier2d in 2D, both with enhanced determinism): game code owns the components, the engine's og.physics system steps them after the game's systems and writes the dynamic bodies back. |
| [`atmosphere`](atmosphere.md) | `og.atmosphere.*` | kept | Fog, a procedural sky, the day-night cycle, weather, the wind that sways vegetation and water, declared as resources and components (the E5 round). Everything is off by default: a world that sets none of them draws exactly as before. |
| [`anim`](anim.md) | `og.anim.*` | kept | Skeletal animation over og-anim: glTF skins and clips (loaded with og.load.animation), blend-tree graphs with layered state machines, root motion, two-bone and FABRIK IK and blendshapes, evaluated by the engine's og.anim system after the game's systems and before physics. |
| [`tween`](tween.md) | `og.tween.*` | kept | Tweens: animate any numeric field of a component (og.Transform.position, a game's own f32) to a target over time with an easing curve, a delay, repeat and yoyo, in sequences or in parallel. Advanced by og.tween after the game's systems, before og.anim and physics, by scaled time unless unscaled. |
| [`tilemap`](tilemap.md) | `og.tilemap.*` | kept | Tilemaps: layers from a checked file.tileset TS atlas, loaded from a cooked map (checked file.tilemap TS, Tiled .tmj or LDtk .ldtk) with og.load.tilemap and placed by og.Tilemap; og.tilemap.create and resize make and change maps in game code. |
| [`audio`](audio.md) | `og.audio.*` | kept | Sound: inside the tick, the engine's og.audio system (after the game's systems and physics) turns AudioSource changes into a deterministic, time-stamped command stream; outside the tick the host's mixer renders it (an AudioWorklet on the web, the output device, a capture or a null sink natively). |
| [`nav`](nav.md) | `og.nav.*` | kept | Navigation: grids (A* and jump point search, eight-way moves that never cut a blocked corner, line-of-sight smoothing), flow fields for crowds, and navmeshes (cooked, or built at runtime) with funnel paths and box obstacles that rebuild only the tiles they touch. |
| [`ai`](ai.md) | `og.ai.*` | kept | Enemy and NPC AI as checked data: file.behaviour TypeScript declarations compile trees and tick state machines to native fields. Perception checks sight cones against physics occlusion and hears spatial sounds and game noises; each agent has a blackboard. |
| [`story`](story.md) | `og.story.*` | kept | Dialogue and quests from checked file.dialogue and file.quest TypeScript declarations, cooked as native data. Stable node, line and choice IDs are independent of localization keys. Quests declare stages, objectives and rewards. External Yarn Spinner 3.2.2 uses an explicit input.text import, never automatic scanning. |
| [`platform`](platform.md) | `og.platform.*` | kept | Platform services as host capabilities: the cloud store and save slots, leaderboards, telemetry, Origin Point, AI marks, purchases, ads, achievements, the player, friends, invites, share and rooms. Game code names a method and its arguments, never a credential, a URL or a cookie: the host owns them; every id it hands back is opaque and scoped to this game. Calls are asynchronous: each returns a request number at once and its answer arrives at the start of a later tick (og.PlatformAnswered, og.platform.result), host input like key presses. A host that lacks a method answers it `unavailable`. |
| [`i18n`](i18n.md) | `og.i18n.*` | kept | Localization: one file.strings declaration per locale under content/, messages in a subset of ICU MessageFormat with CLDR plural rules, and the locale the host asks for. The active locale is the best loaded table for the request, else the base table; a key it lacks is read from its fallback table, then the base. It also sets the UI's font chain and direction. |
| [`ui`](ui.md) | `og.ui.*` | kept | The engine's UI: TSX components (@origingame/engine/ui's mount) styled by Tailwind classes and the style pack, laid out and drawn over the frame, and answered to the probe (dump_ui, touch_move_to). These calls configure it and read it back. |
| [`systems`](systems.md) | types and tokens | kept | Systems the engine ships as TypeScript: game-level code, listed explicitly in GameDesc.systems like the game's own, so the schedule (and dump_world) shows them and nothing runs that the game did not ask for. |
| [`commands`](commands.md) | `og.commands.*` | dropped | The command channel: actions scheduled on simulation time, from `og run --commands "t:action[:args];..."` or the probe's commands(). |
| [`probe`](probe.md) | types and tokens | kept | The probe: RuntimeProbe protocol 5 (origin/packages/contracts/src/probe.ts) served natively, plus the engine's wire members. |
| [`terrain`](terrain.md) | `og.terrain.*` | kept | Terrain: a height field over a rectangle, cooked by og cook from a TS file.terrain declaration (an image, noise or evaluated heightfield source, control maps, up to sixteen layered materials and a holes map) into tiles; drawn as a clipmap around the camera, collided with as a height field, baked into navmeshes. |
| [`stream`](stream.md) | `og.stream.*` | kept | Named groups load one pack each, by priority within a memory budget (256 MiB by default). Resident means pack readiness. Models stay hidden until textures are prepared; StreamStatus.ready is presentation-only and nondeterministic. Physics keeps proxy meshes, independent of texture progress. |
| [`video`](video.md) | `og.video.*` | kept | Footage authored with file.video in TypeScript, cooked by ffmpeg, loaded with og.load.video and played by og.VideoPlayer. A clip (the default: at most 256 frames) is cooked into a texture array; a stream (`stream: true`: any length) is cooked to AV1 and decoded as it plays. The frame is a Texture; the sound plays on the entity's og.AudioSource in step with it. |
| [`debug`](debug.md) | `og.debug.*` | dropped | Debug draw: lines, boxes, spheres, arrows, axes and labels in the world, each for a number of ticks, drawn over the finished frame (after every post effect, before the UI); and the debug views the command channel's `debug:` actions switch. |
| [`assert`](assert.md) | `og.assert.*` | dropped | Assertions: properties of the game checked as it runs, after the Antithesis SDK's (AssertKind: always, sometimes, reachable, unreachable, each). Each is identified by its name and the place it is called from, which the build records. Nothing an assertion does reaches the world, so a run with them hashes and replays as one without. Violations and coverage go through the probe (`assertions`) into `og run --json` and `og test`. |
| [`path`](path.md) | `og.path.*` | kept | Paths: a curve through or controlled by points, measured by arc length so a parameter t from 0 to 1 moves along it at an even speed. catmullRom passes through every point (tension 0.5), bezier chains cubic segments `p0 c0 c1 p1 c2 c3 p2 ...`, linear joins straight segments; a closed path joins its last point to its first and wraps t. |
| [`timeline`](timeline.md) | `og.timeline.*` | kept | Timelines: cutscenes and sequences as typed file.timeline declarations under content/. Keyed tracks cover camera, transform, animation, audio, captions, events, shots and scenes. og cook builds native tracks; og.load.timeline loads the canonical content ID. See animation and file:timeline. |
| [`net`](net.md) | `og.net.*` | kept | Deterministic lockstep over a room (docs/design.md §2.10): every peer runs the whole world, and what crosses the room is each player's input, one per tick, and the world's hash. Turn it on with og.Lockstep and join a room (og.platform.roomJoin); once the room holds `players` peers the authority (the lowest peer number) starts, and tick t runs when every seated player's input for t is in (in rollback, on a prediction re-run when the input differs). |
| [`shared`](shared.md) | `og.shared.*` | kept | Shared worlds: visitors of one large world see each other's avatars in real time, near them only. Each client runs its own world; a world server (`og world-server`) holds the avatars, checks each move and sends each client those near it. The game marks its avatar with og.SharedAvatar and moves it as always; every other visitor in sight is an og.RemoteAvatar entity the engine spawns, moves and despawns. |
| [`art`](art.md) | types and tokens | kept | A game's art contract: what every asset of its style holds, as seven resources a look sets beside the render ones (docs/art.md §2.1): ArtPalette, ArtPixels, ArtTexels, ArtShape, ArtLine, ArtMotion and ArtCamera. A style is the assets, not a filter on the frame. |
| [`motion`](motion.md) | `og.motion.*` | kept | Generated motion for humanoids: the engine's motion model moves a character between the game's goals, on the GPU. Give a humanoid og.Skeleton an og.MotionAgent, then queue goals (og.motion.walkTo, og.motion.jump, og.motion.pose, ...). |
| [`destruction`](destruction.md) | `og.destruction.*` | kept | Props that break (walls, crates, pillars, glass) and 2D ground that carves. A model cooked with a `fracture { }` block is cut by the cook into Voronoi chunks bonded where they touch; hits, og.destruction.damage and explode break an og.Destructible's bonds, each unanchored island falls as one body, and a stress solver (NVIDIA Blast's) breaks what can no longer carry its own weight. og.PixelTerrain is 2D ground of cells. |
| [`vfx`](vfx.md) | `og.vfx.*` | kept | Composed effects: one effect (a fireball's impact, a heal, a level-up) as a timeline of elements in an `.ogt` of `type vfx`: emitters, meshes, trails, beams, lights, decals, glows, flipbooks, shakes and sounds. |
| [`sight`](sight.md) | `og.sight.*` | kept | Fog of war: which ground each team sees now and has seen before. An og.SightMap lays a grid over the og.Volume on its entity; every og.Sight is a team's eyes, and og.SightBlocker regions stop them. The map is world state, cast again as each tick starts: saved, restored and hashed with the world, the same on every host. The game asks it about a point (og.sight.visible, og.sight.explored), and the renderer draws the same cells over the frame. |
| [`flow`](flow.md) | `og.flow.*` | kept | A game's flow as a Yarn dialogue of segments: film, cutscenes, levels and minigames. A node with an `og-flow: <timeline>` header plays that segment on a deck: footage, a live scene (a `scene` track) or both. Its choices open at `flow.open`, lock at `flow.lock` and are taken at the cut (`flow.cut`, else the end), else the `#default`. A playable segment stops at `flow.hold` until og.flow.advance takes a hidden `#outcome:<name>` choice. |
| [`scene`](scene.md) | `og.scene.*` | kept | Change scenes with assets fetched first: og.scene.preload requests a scene's streaming groups, og.scene.progress reports their arrival, and og.scene.change replaces the current scene as the next tick starts. A scene is a GameDesc.scenes name (`level2`) or canonical file.scene content id (`base/scenes/level2`). |
| [`loading`](loading.md) | `og.loading.*` | kept | The loading screen: who draws it (project.ts `publish.loading`: engine, game or page), the stages the game reports while it loads, and when it is done. Every host answers the same calls; the web page also tells whatever page frames or hosts it (window events and, when framed, postMessage), so any site can draw its own loader. |
| [`play`](play.md) | `og.play.*` | kept | Mechanisms that span domains, as facts. An og.HitVolume (on a bone through og.BoneAttachment, or on any entity) is swept from where it was at the last tick to where it is, so a fast swing meets what it passed through, and strikes each og.HurtVolume of another team, and the colliders on its layers, once a swing: og.HitEvent. og.play.candidates lists what is in range, in a cone and in sight. Damage, prompts and choices are the game's rules. |
| [`combat`](combat.md) | types and tokens | kept | The records og.systems.combat carries out: an og.Attack plays og.AttackMove entities tick by tick (startup, active, recovery), drives its og.HitVolume entities in the active window, and takes an og.AttackLink when its input action's buffered press lands in the link's window (og.input.consume); an og.HitEvent of a move's swing or an og.Projectile takes its damage from the target's og.Health, unless its og.Hurt says it is invulnerable, and is reported as og.Damaged and, once, og.Died. Every number is the game's: a zero window, damage or invulnerability does nothing. |
| [`interact`](interact.md) | types and tokens | kept | The records og.systems.interact carries out: an og.Interactor focuses one og.Interactable among og.play.candidates by its own rule (nearest, most centred, priority then nearest), uses it on a press of its action or after holding it, and reports og.Interacted. Using an og.Door opens or closes it and its navmesh link, an og.Toggle flips, an og.Grab is picked up and later thrown. Every number is the game's: a zero range focuses nothing. |
| [`table`](table.md) | `og.table.*` | kept | A game's data as rows keyed by id: items, abilities, recipes, loot, waves, levels. Author file.table in content/**/*.ts against a component declared in project.schemas. Cook validates rows and references and packs native reflected values. load installs these rows; replace installs generated runtime values. Stable handles survive reordering, reload and saves. The values are the game's: nothing here picks, sums or balances them. |
| [`inventory`](inventory.md) | `og.inventory.*` | kept | The records og.systems.inventory carries out, and the moves a game makes with them: an og.Container holds `slots` slot entities (its children, each an og.ItemStack of a table row and a count), a stack holds at most its row's stack limit, read from the item table, and every move, split, merge and craft is all or nothing. An og.Pickup lying in the world is taken by the og.Collector that enters its sensor. Every number is the game's: a container of 0 slots holds nothing. |
| [`abilities`](abilities.md) | `og.abilities.*` | kept | The records og.systems.abilities carries out: an og.Ability is used on a press of its action or by og.abilities.use when it has a charge, its lock has run out and its pool holds its cost; its charges return one by one. An og.Status on a target merges with one of the same kind by its stacking rule and runs out after its ticks; an og.Stat's value is its base with the og.Modifier entities naming it applied in a fixed order. Every number is the game's: an ability of 0 charges is never used. |

## Units

Field units the docs and Tune use.

| Unit | Means |
|---|---|
| `m` | metres |
| `s` | seconds of simulation time |
| `ms` | milliseconds |
| `rad` | radians: every rotation, heading, yaw, joint or steering limit, arc, uv rotation and cone; a cone is given as its half-angle |
| `deg` | degrees: a field of view (the full angle), a slope, the sky's angles (the sun's path, disc sizes, the aurora), camera roll, and the angles of post effects, styles and the art contract |
| `hz` | ticks or cycles per second |
| `ticks` | fixed simulation steps |
| `tick` | a tick number (og.Time.tick) that marks when something happened or happens; a world checked against a fresh start (playtest restart) compares it as an offset from its own og.Time.tick, and 4294967295 (never written) as is |
| `cssPx` | CSS pixels of the play surface, origin at the top left |
| `px` | pixels of an image, origin at its top left |
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
| `m3` | cubic metres |
| `pascal` | pascals: newtons per square metre, a stress |
| `newtonSecondPerM2` | newton-seconds per square metre: an impulse spread over an area |
| `db` | decibels |
| `evPerSecond` | exposure stops (EV) per second of simulation time |
| `bpm` | beats per minute |

[File and component reference](reference.md)
