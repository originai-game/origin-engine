---
name: og-game
description: Building a game on Origin Engine — the entry point: the project layout and project.ogt, components, resources and systems on the ECS, the game entry (GameDesc, commands, facts), scenes as .ogt text, assets cooked from assets/ to cooked/ (glTF and MagicaVoxel models), the og.load loaders and streaming groups, input (keyboard, pointers, gamepads, pointer lock), saving progress (world snapshots and records), checking a run (describe, expect, traces, write-back, budgets), and the map of the other skills with the modules each teaches (2D, animation, rendering, physics, networking and the rest). Use first, and when creating, structuring or extending a TypeScript Origin Engine game (a project with a project.ogt; not a Godot project.godot).
---

An Origin Engine project is plain files: TypeScript in `src/`, source assets
in `assets/`, text scenes, and a manifest. `npm create @origingame/game`
produces it; everything after that is code and text, never an editor. This
skill is where to start: what a game is made of, and which skill to read
next for each part.


## What works

`examples/<name>` in this package's skills is a game in the engine's
source repository (https://github.com/originai-game/origin-engine/tree/main/examples),
not installed with the package; read one there when a skill points to it.
What a game starts from is installed: the archetypes of
`@origingame/create-game` (`npm create @origingame/game -- --archetype <name>`,
or Origin's `engine_scaffold`).

Three games were written from this package alone: `examples/lantern-vale`
(a 3D top-down adventure), `examples/hollowbell` (a 2D platformer) and
`examples/kestrel-stair` (a 3D third-person action-adventure). What they
asked for the documented way worked the first time: a Tiled map
with a blob47 auto-tiled terrain, Tiled's parallax and objects with
properties (og-2d §4); flipbook frame events driving footsteps, strikes and a
bell; texture arrays as world sprites, screen sprites and emitter layers;
streaming groups of `.vox` models and textures loading as the camera
neared them (§6); a look applied before `start` and a second one at a tick
for the results (og-taste §13); `og.physics.overlap` on a layer and with sensors;
a character controller shoving a pile of bodies (og-physics §2); a skinned glTF on
twos (og-anim §1). In The Kestrel Stair: one clip library retargeted at cook time
onto a Mixamo-named, A-posed character, and at run time
(`og.anim.retarget`) onto an Unreal-named one (its bones detected) and
onto one whose bones only a `type rig` file names, all three playing
through blend trees the first time, the clips' `hit` markers coming
through both retargets (og-anim §4); a slam lunging about a metre by root motion,
`Skeleton.rootMotion` planar for its length; a navmesh baked from the
level's glTF (og-ai §1); the 3D character controller over triangle-mesh
slopes, steps and ledges, refusing a scree too steep (og-physics §2); a cape, four
banners in a gusting wind and a chain with a lamp at its last link (og-physics §2);
`world.snapshot()` and `world.restore()` across a relaunch, bringing back
93 entities with their cloth, paths and minds (§8). `og cook` took their
47, 69 and 62 assets cold in 8.2 s, 3.6 s and 8.5 s, about 0.1 s after a
change, without a warning, and its refusals named the line and the fix.

## Writing a game from this package

The order the games found productive:

1. **`project.ogt`** (§2): name, id, the engine pin, the profile, tick and
   seed. The scaffold writes it (`npm create @origingame/game --profile
   2d` writes the 2D starter, a platformer with sprites, a tilemap and 2D
   physics, as does a premise that says 2D, platformer, side-scroller,
   metroidvania or pixel art; `3d`, the default, the 3D template; an
   archetype brings its own); change it there and nowhere else.
2. **`GameDesc`** in `src/main.ts` (§3): the scene, the look, the systems,
   and from the start a command for each state a test must reach (`play`,
   `warp:<where>`) and facts for what it must check.
3. **Assets by script**: `tools/make-assets.*` writes every texture,
   tileset, map, flipbook, emitter, sound and look deterministically (every
   game and archetype does), with import settings beside each source
   (§5, og-2d §1, og-render §2). Rerun it after a change; never hand-edit its outputs.
4. **Cook** (`og cook`, og-build §1): read every refusal, it names the line
   and the fix.
5. **In 3D, the navmesh before the minds** (og-ai §1): bake it from the
   level (its glTF's `model` line), load it in `start` and check
   `og.nav.nearest` at every spawn and route point; only then write the
   behaviours.
6. **Run with commands** (og-build §1, §4): `og run . --headless
   --commands "0.5:screenshot:x.png;2:key_down:Space;3:screenshot:y.png;3.5:quit"`
   (every step starts with its time in seconds), then open each frame and
   say what it shows. Do this after every change that shows on screen.
   Add `describe:frame.json` beside a screenshot to get the frame as data:
   the visible entities nearest first, each with its screen box, pick point,
   depth and the fraction hidden; the UI elements with rects and text; the
   camera (§9).
7. **Playtests with reference frames** (og-build §3): `tests/*.og.ts`
   asserting on facts, `get:` answers and dumps, or written as `expect:`
   steps alone, which `og run` itself checks (§9), one per feature, and a
   frames playtest comparing screenshots with `references/<tier>/`, its
   frame reads inside `t.frame`: in a sandbox without a renderer each
   screenshot is reported "no renderer, frame unchecked" and passes while
   every other check runs, and the frames are verified where there is a
   renderer (the author's machine, the browser-check sandbox, an executor
   with a GPU).
8. **Polish** (og-build §5): `og polish` until its findings are gone; each
   names the section that fixes it (og-taste §10, og-feel §13).

Mistakes the games made, or would have, before reading the right page:

- **The resource handle.** A system reads or writes a resource only when
  it lists it among its `reads` or `writes` (og-core §4: undeclared, it
  throws naming the system and the record). A field read or write is a
  typed-array access whatever the resource's size, and `world.resource(R)`
  answers the same handle wherever it is called: keep the run's state in
  one resource and write its fields where they change, never through a
  plain copy (og-core §4 has the cost model: a field read or write about
  0.03 µs, the first read after an engine call about 2 µs while the record
  is fetched again, writes sent in one call before the next engine call
  and when the system returns).
- **A look replaces resources whole.** `GameDesc.look` and
  `og.render.applyLook` set every field of each resource the look names
  (the rest to their defaults), so a per-game value written before the
  look is lost: write it in the look, or in `start` after it (og-taste §13).
- **Where probe outputs go.** `og run` writes screenshots and dumps under
  `og-out/` (`--out` chooses), `og test` each playtest's under
  `og-out/test/<name>/`, and a playtest reads them back by the name its
  step gave (`run.json("shop.json")`). Give every step's file its own
  name, so a check reads the step it means (og-build §1, §3).
- **An agent off the mesh.** Place every navmesh agent at its
  `og.nav.nearest` point (idle or seeking a target on the mesh), and have a
  playtest read each agent's position and `status` a few ticks in: still
  where it was put, its height too, and not `NAV_UNREACHABLE` (og-ai §2).

## 1. The skills

| Skill | Read it for | Its modules (`og api show <module>`) |
|---|---|---|
| **og-core** | the rules every line obeys: TypeScript only, the fixed tick, Results, archetype views, reflection | math, reflect, world, system, rng |
| **og-game** (this one) | the project, components and systems, scenes, assets, loading and streaming, input, saving, checking a run | game, transform, input, load, stream |
| **og-2d** | sprites, billboards, flipbooks, texture arrays, tilesets, maps and Tiled | tilemap |
| **og-anim** | skeletal clips, blend trees, IK, look-at, foot locks and spring bones, markers, retargeting, 2D skeletons, cutscenes and paths | anim, path, timeline |
| **og-render** | effects, particles, light and shadows, reflections, atmosphere, water, terrain, shaders, post passes, render targets, run-time geometry, debug draw, footage | render, atmosphere, terrain, debug, video |
| **og-physics** | bodies, contacts, forces and queries; joints, the character and platformer controllers, vehicles, ropes, cloth, fluids, ragdolls | physics |
| **og-net** | two or more players in lockstep over a room, and a desync | net |
| **og-build** | the loop: cook, run with a script, read frames and dumps, hot reload, playtests, `og polish` (og-build §5: the one list of what it scores), shipping, performance, `og render` videos | commands, probe |
| **og-feel** | how much: hit stop, shake, squash, easing, floating text, the camera, frame rates, sorting, parallax, controller tunables | time, tween, camera, systems |
| **og-ui** | screens and HUDs: templates, widgets, focus, phones, UI styles and motion | ui |
| **og-taste** | the look: a style from the premise, composition, colour, type, motion, feedback, atmosphere by intent, the done checklist, looks as data and the art contract | art |
| **og-model** | a 3D model built as code from a reference image: the detail inventory, eight gated passes, the generators and their fields, the `type model` asset, budgets, the review frames and `og compare` | shapes |
| **og-ai** | enemies and NPCs: navigation and minds as data | nav, ai |
| **og-audio** | sound: events, presets, UI sounds, the mix, music, space | audio |
| **og-i18n** | languages: string tables, messages, fonts, line breaking, right to left | i18n |
| **og-platform-sdk** | the platform from inside the game: saves, settings, leaderboards, telemetry, points, AI marks | platform |

Beside them, six craft skills teach no module and name no API of their
own; they point to the skills above for every call:
**architecture-discipline** (the project's layers, when to split, the first
playable, loud failure, tuning in data), **component-forge** (a unit built
as code from a measured reference: the route, decomposition, gates,
assembly), **level-texturing** (a gray level painted from views with `og
bake projection`), **localization** (the discipline beside og-i18n),
**recipe-craft** (the retrieval ladder for example code, vendoring and
distilling) and **visual-acceptance** (judging frames against references,
baselines and the play camera).

`og api show <module>` names the module's skill, as this table does. When
two skills seem to touch the same thing, the one in this table owns it and
the other points there.

## 2. The project

| Path | What it is |
|---|---|
| `project.ogt` | The manifest: `name`, `id`, the pinned `engine` version, `entry`, `profile` (`2d` or `3d`), `tick`, `seed`, `scenes`, the first `scene`, the game's `look`, the `screens` `og style check` captures (og-build §8), `publish` settings |
| `src/main.ts` | The game entry: its default export is `og.game({...})` |
| `assets/` | Source assets: glTF and `.vox` models, PNG/JPEG textures, WAV/MP3/Ogg Vorbis/FLAC sounds, TTF/OTF fonts, `.ogt` scenes and looks, and `<file>.ogt` import settings |
| `cooked/` | `og cook` output (`.ogb` blobs and `.cook.ogt`); never committed, the only thing the runtime reads |
| `tests/*.og.ts` | Playtest scripts `og test` runs headless |
| `dist/` | `og build` output |

The engine version in `project.ogt` is a pin. Upgrading it is an explicit
change with its own run, never a side effect: `og upgrade --json` moves the
pin and package.json's dependency to the installed engine, applies the
mechanical rewrites of the versions crossed and lists each line to edit by
hand with its fix (exit 3 while any remain); then `bun install`, `og
check`, `og test`. A newer patch of the pinned minor runs the game as
written. Its `tick`, `seed`, `scenes`,
`scene` and `look` are the defaults of the `GameDesc` fields of those
names: what `og.game({...})` leaves unset takes the manifest's value (else
60, 0, none, `main` and no look). Write each in one place; set in both to
different values, the engine refuses to start and names both. `look
"looks/<id>.ogt"` is also what the cook reads the game's art contract from
(og-taste §14), before any asset cooks, so a game with a look names it here.

`og check` type-checks the project (tsrs, a Rust port of TypeScript 7,
installed with the engine package; where it has no package, tsgo through
`bun x`). `og run`, `og test` and `og build` check before they bundle and
stop on a type error, printing each diagnostic as `file(line,col): error
TS<code>: message`; with `--json` the diagnostics are a `check` event
(`{"event":"check","ok":false,"diagnostics":[{"file":"src/main.ts","line":12,"col":7,"code":2322,"message":"..."}]}`)
on the same stream as the run's other events. `og dev` checks each save
beside the rebuild: the reload still happens, the `reload` event carries a
`check` object of `ok`, `ms` and `errors` (`null` while the check runs)
and a `check` event with the diagnostics follows; `og test --json` ends
with a `test` event whose `check` object has the same three. Read the
diagnostics and fix them before anything else; `--no-check` skips the
check (never for `og build --publish`). A `2d` project's publish build
leaves the 3D world out (terrain, planar water, scattered copies, the 3D
fluids, volumetric fog: `og.render.capabilities(world).world3d` is false
there), so `og build --publish` refuses one whose scenes hold `Terrain`,
`TerrainScatter`, `Water`, `WaterSurface` or `Fluid3D`, naming each
`scene.ogt:line`, and there `og.terrain.*` and a `SHAPE_HEIGHTFIELD`
collider answer a refusal naming `world3d`: remove them, or make the
project `profile "3d"`. A created project's
`tsconfig.json` resolves `@origingame/engine` (the generated
`types/index.d.ts`) and `@origingame/engine/playtest` from `node_modules`.
An example inside the engine repository has no `node_modules`: its
`tsconfig.json` maps both through `paths` to
`packages/engine/types/index.d.ts` and `packages/engine/src/playtest.ts`
(with `packages/engine/src/env.d.ts` and a `host.d.ts` for the `node:`
modules and globals its bun scripts touch), and `og check .` reads it
(`examples/hollowbell`, `examples/lantern-vale`).

## 3. Components, resources, systems, the game

```ts
import { og, defineComponent, defineSystem, f32, Vector3 } from "@origingame/engine";

export const Health = defineComponent("Health", { hp: f32(100, { min: 0, max: 1000, tunable: true }), max: f32(100) });
export const Velocity = defineComponent("Velocity", { v: Vector3() });

export const Move = defineSystem("Move", { reads: [Velocity], writes: [og.Transform] }, (q, time) => {
  for (const a of q.archetypes()) {
    const p = a.column(og.Transform).position;
    const v = a.column(Velocity).v;
    for (let i = 0; i < a.count * 3; i++) p[i] += v[i] * time.dt;
  }
});

export default og.game({
  tick: 60,
  scenes: { main: "scenes/main.ogt" },
  systems: [Move],
  commands: { spawn_wave: (world, args) => spawnWave(world, Number(args[0])) },
  commandHelp: { spawn_wave: { usage: "<count>", doc: "Spawns a wave of <count> enemies now." } },
});
```

- A **component** is plain data per entity; a **resource** is a reflected
  singleton (tuning tables, game state) read with `world.resource(token)`.
  Engine-owned ones are tokens on `og`: `og.Transform`, `og.Time`, `og.Rng`,
  `og.Input`, `og.Camera`.
- A **system** is a function over its matched archetypes, once per tick
  (og-core §4 has the rules). Keep each system to one job; share state through
  components and resources, never module variables.
- **Entities** are spawned with `world.spawn({ name, parent, position })` and
  addressed by name everywhere outside the hot loop: `world.find(name)`, the
  probe's ids, `teleport`, `set`, `dump_entity`. Give every entity a person
  will ask about a stable name.
- **Commands** in `GameDesc.commands` are snake_case actions the command
  channel and playtests can call (`0.5:spawn_wave:3`), with their usage in
  `GameDesc.commandHelp` for `help` (og-build §1). Add one for each state a
  test needs to reach quickly, instead of playing there.
  - **Arguments are colon-delimited**, the fields after the name, each a
    string (`Number(args[0])` to read a number); a space is part of its
    field, not a separator, and `\:` writes a colon inside one. Two
    arguments, and the usage that says so:

    ```ts
    commands: { records: (world, args) => setRecords(world, Number(args[0]), Number(args[1])) },
    commandHelp: { records: { usage: "<berries>:<seconds>", doc: "Sets the isle's records." } },
    ```

    `0.5:records:12:34.5` hands `args` `["12", "34.5"]`; `records:12 34.5`
    would hand one argument, `"12 34.5"`.
  - **Reserved names.** The channel's core actions are not a game's to
    take, and the game refuses to start on one (`'pause' is a core action;
    register under another name`): `screenshot`, `listen`, `touch_move_to`,
    `touch_move`, `touch_start`, `touch_end`, `touch`, `scroll`, `gamepad`,
    `key_down`, `key_up`, `type`, `compose`, `commit`, `move_to_text`,
    `teleport`, `frame_entity`, `dump_world`, `dump_entity`, `dump_ui`,
    `get`, `set`, `seed`, `pause`, `resume`, `advance`, `describe`,
    `expect`, `save_scene`, `quit` and `help`. The actions the engine's
    modules register are refused the same way (`'view' is a registered
    engine action; register under another name`), in every build:
    `bake_probes`, `cover`, `debug`, `dump_ai`, `dump_desync`,
    `dump_lanes`, `dump_nav`, `dump_platform`, `dump_pulse`,
    `dump_render`, `dump_room`, `dump_stream`, `lanes`, `locale`,
    `terrain_stream`, `timeline`, `turntable` and `view` (`help` marks
    them `(registered)`). The engine's `pause`
    freezes the tick from outside the game; the game's own pause (its menu
    open, `Game.paused` set) is a game action under its own name,
    `open_pause`.
- **Facts** (`GameDesc.facts`) return a small JSON object for the probe's
  `state()`: score, phase, lives. Tests assert on facts and dumps.

`og api show game` is `og.game` and every `GameDesc` field;
`og api show transform` is `og.Transform`, parents and the matrix helpers.

## 4. Scenes are text

A scene is an `.ogt` file under `assets/`, cooked like any asset:

```text
type scene
version 1
---
name "Main"
entity "ship" {
  model "models/dropship.glb"
  Transform {
    position 0 1.5 0
  }
  entity "light" {
    Transform {
      position 0 3 0
    }
  }
}
```

Each `entity` has a name, optional `model` (a glTF under `assets/` whose
default scene becomes the entity's children), component blocks keyed by the
component's reflected name with one line per field, and nested child
entities. Fields not written take their defaults. Edit scenes as text and
diff them like code; the cook checks field names and lane counts against the
registered schema and refuses a model path that does not exist.

## 5. Assets: assets/ to cooked/

- Put sources in `assets/`; `og cook` turns them into `cooked/`: glTF into
  meshopt-compressed meshes with LODs, textures into KTX2 Basis Universal
  (ETC1S for colour, UASTC for normal and linear maps), WAV, MP3, Ogg
  Vorbis and FLAC into QOA, fonts into MSDF atlases, scenes into component
  tables. Import settings live next to the source as `<file>.ogt`
  (`texture { kind normal }`,
  `font { charset latin chars "开始游戏" }`). The prose here writes a
  block on one line for short; in a file, as in every `.ogt`, a `{` ends
  its key's line and a `}` stands alone on its own (the parser refuses
  `'{' must end its line`):

  ```text
  type import
  version 1
  ---
  texture {
    kind normal
  }
  ```
- At load a cooked texture reaches the GPU block-compressed in the device's
  format with its mips (BC7, and BC5 for normal maps, on desktop GPUs; ASTC
  on Apple and most Android; ETC2 and EAC elsewhere), a quarter of RGBA8's
  memory. Give textures sizes in multiples of 4 (powers of two are best): a
  size that is not, pixel art (`kind pixel`) and fonts stay RGBA8.
- Declare the characters a font must show (`chars`) rather than cooking a
  whole CJK charset: the full set is several times the size of the font.
- MagicaVoxel `.vox` files are models: the cook meshes every model of the
  file (greedy quads, flat-shaded, the palette as vertex colours) and its
  scene graph into a glTF, z-up turned y-up, each model about its centre
  (MagicaVoxel's pivot, with or without a scene graph), with one white
  rough material your `og.MaterialStyle` or own material replaces. Load
  and place it as any glTF: `og.load.gltf(world, "models/hut.vox")`, or a
  scene's `model "models/hut.vox"`. The `voxel-worlds` look (og-taste §12) suits
  them. Its import settings (`models/hut.vox.ogt`) set the size and the
  origin; without them a voxel is one metre and the origin is the editor's
  (a character 16 voxels tall would be 16 m, its feet 8 m below its
  position):

  ```text
  type import
  version 1
  ---
  model {
    voxScale 0.1      # metres per voxel (default 1)
    voxOrigin bottom  # center (default: the editor's origin) | bottom (the lowest voxel on y = 0)
  }
  ```

  A `.vox` with frames (MagicaVoxel's animation) cooks to clips that swap
  the frames' models: play them as skinned clips (og-anim §3). `voxFps 8` sets the
  rate (default 10), `voxClip walk 0 3` names frames 0 to 3 (repeatable;
  without it one clip `frames` holds the playback range set in the editor,
  else every frame; a node marked to loop repeats its keys over the file's
  frames). Each frame is centred on
  its own pivot, as in the editor: give the frames one height (or pad them)
  for a character whose feet stay on the ground.
- An `.ogt` of `type model` is a model written as text: og-geom parts by
  their `og.render` names, placed, with materials and pivots (og-model §9).
- Reusable production assets come through the platform's asset catalog into
  `assets/`; record each file's licence next to it.

## 6. Loading

`og.load.*` returns a handle at once and completes at a tick boundary; under
`og run` and in replays every load completes before the next tick, so the
tick it lands on is deterministic. Paths are asset ids (the path under
`assets/`). Check `og.load.status` (or the `AssetReady` / `AssetFailed`
events) before using an asset, and spawn a loaded scene or model with
`world.instantiate(asset, { name, position })`. The scene in
`GameDesc.scene` is spawned for you when the world starts. `og api show load`
lists every loader and what each returns.

**Streaming groups** (`og api show stream`) keep a big world's assets off
the GPU until the player is near them. A scene names glTF models (no skins
or morph targets), textures, texture arrays, sounds, environment maps,
palettes, looks and colour LUTs (`.cube`) that load and unload together,
and what loads them:

```
stream "harbour" {
  asset "models/harbour.glb"
  asset "textures/harbour-sign.png"
  asset "audio/gulls.wav"
  distance 0 0 -200 80     # the camera within 80 m of (0, 0, -200)
  unload 120               # kept until past 120 m (1.25 x the radius by default)
  priority 2               # higher loads first and is evicted last
}
stream "cave" {
  asset "models/cave.glb"
  volume "cave-mouth"      # the camera inside that entity's box
}
stream "boss" {
  asset "models/boss-arena.glb"   # no trigger: og.stream.request only
}
```

Load and spawn grouped assets as usual: until the group is resident a model
draws its proxy (its coarsest LOD, untextured), a texture is a grey 1x1 and
a sound plays silence of its own length; then the real ones take over the
same handles. Physics always collides with the proxy, so the simulation and
the world hash never depend on what is resident. Texture arrays,
environment maps, palettes, looks and LUTs have no stand-in: load them once
the group is resident; when it unloads their handles wait (the sky lights
the scene, `og.render.sprite` of the array is refused, an `og.Sprite` or a
particle emitter of it draws nothing, `PostEffects.lut` grades nothing) and
come back with the group. One the frame still uses when its group unloads
is kept, with a warning, until the frame stops using it, and its bytes
count in the budget meanwhile (`og.stream.budget().kept`). Groups are admitted by
priority within `og.stream.setBudget` (256 MiB by default); a lower-priority
group is evicted to make room for a higher one. `og.stream.status`,
`og.stream.budget`, `og.StreamEvent` (state `STREAM_RESIDENT` and the rest)
and the probe's `dump_stream` say where each group is. `og build --publish`
ships each group as its own file, fetched only when the group loads.

There is no unload call for a single asset: memory comes back when a
group unloads (its trigger lets go, or `og.stream.release(world, "boss")`
after `og.stream.request`). Keep the handles you have; they wait and come
back with the group, never load them again. To let a kept asset go, stop
using it (`og.Environment.map` or `PostEffects.lut` to 0, another look).
`StreamStatus` (`og.stream.status(world, "harbour")`) has the group's
`state`, `trigger`, `memory`, `kept`, `loads` and `reason`; `StreamBudget`
has `budget`, `used` (kept bytes included), `kept` and `peak`. At most one
group is put in place per tick boundary, and a group larger than the whole
budget fails. A chunked sound (og-audio §5) may join a group: its first
chunk comes in the pack and its other chunks are read ahead of its
playheads only while the group is resident.

## 7. Input

`world.resource(og.Input)` is the keyboard, the mouse and the first touch
(`down("KeyW")`, `pressed`, `axis`, `buttonDown`). Two more resources carry
what one pointer cannot: `og.Pointers` (pointer 0 is og.Input's, 1 to 4
further touches: `down(id)`, `pressed(id)`, `position(id)`, `start(id)`)
and `og.Gamepad` (up to four pads: `connected(pad)`, `down("a", pad)`,
`pressed`, `released`, `axis("leftX", pad)`; sticks are y down). All three
are world state, so playtests and replays reproduce them; script them with
`touch:<id>:<down|move|up>:<x,y>` and `gamepad:<a+b+...|->[@pad][:lx,ly][:lt,rt]`
(og-build §1). The fields and methods are `og api show input`.

A press happens somewhere inside a tick, and the tick reads it at its
start: `og.KeyDown`, `og.KeyUp`, `og.PointerDown` and `og.PointerUp` carry
`at`, when it happened in ticks from the start of the tick that reads it
(-1 to 1; from the browser event's time in a page, from the step's time
in a command script; 0 for the edges a tick's end raises and for gamepad
presses, which are polled). Timing that matters to the millisecond (a
rhythm game's judgement) uses `time.elapsed + e.at * time.dt`; everything
else ignores it. The stamp is world state, so a replay reproduces it.

Mouse-look (a first- or third-person camera turned by the mouse) takes
the pointer lock and the mouse's relative movement, as browser games do.
Set `og.PointerLock.wanted`; the web shell locks the pointer on the next
click on the canvas (browsers need the gesture) and reports `locked`
(Escape releases it: show "click to look" while `wanted && !locked`).
`og.Pointers.movement` is the mouse's movement this tick in CSS px
(movementX/Y summed, y down, zero next tick); it keeps coming while
locked, when the positions stand still.

```ts
const Look = defineSystem("Look", { reads: [og.Pointers], writes: [og.PointerLock, Aim] }, (_q, _time, world) => {
  const lock = must(world.resource(og.PointerLock));
  lock.wanted = !must(world.resource(Game)).paused; // release it for menus
  if (!lock.locked) return;
  const m = must(world.resource(og.Pointers)).movement;
  const aim = must(world.resource(Aim));
  aim.yaw -= m.x * 0.0025; // radians per CSS px
  aim.pitch = Math.max(-1.4, Math.min(1.4, aim.pitch - m.y * 0.0025));
});
```

Native runs are headless and never lock: script it with
`pointer_lock:on` and `mouse_move:dx,dy` (or inject `{ kind: "pointer",
action: "move", x, y, button: "none", movementX, movementY }` and
`{ kind: "pointerLock", locked: true }`), which replays like any input.

## 8. Saving progress

A save is either the whole world or a record the game writes; both go in
the platform's store (og-platform-sdk §2), which takes any JSON value up
to 64 KiB a key: a record in one key, the world in a slot
(`og.platform.saveSlot`, any size up to 512 KiB).

- **The world** (`world.snapshot()`, `world.restore(save)`): every entity,
  component, name, resource (og.Time and og.Rng too), pending event and
  module state (physics, animation, tweens, tile edits), as text
  (`og-save:1:...`), in every build including publish and alike on both
  hosts. Restoring brings that world back to the hash it had; the session
  stays (input, the player's volumes, the platform's calls, the language,
  the command channel's scheduled steps). Use it where the moment itself
  is the save: a puzzle mid-move, a sandbox, a run that resumes exactly.

  ```ts
  // Save (a lantern lit, Save and quit): the world, and what the title shows of the slot.
  const s = world.snapshot();
  const asked = s.ok ? og.platform.saveSlot(world, { slot: "slot1", text: s.value, meta: { name, chimes } }) : s;
  if (asked.ok) run.saveRequest = asked.value;              // one request for the whole slot
  // The title, on its first tick: the meta alone, no parts read.
  run.slotRequest = must(og.platform.loadSlot(world, { slot: "slot1", metaOnly: true }));
  // Continue: the text, once loadSlot's answer landed.
  run.loadRequest = must(og.platform.loadSlot(world, { slot: "slot1" }));
  const back = world.restore(answer.data.text);            // in a system: applies when the tick ends
  if (!back.ok) toast(back.reason);                        // a save from a build whose types changed names them
  ```

  How big a save is, and so where it goes: a world of a few entities
  snapshots to about 11 KiB of text (most of it the table of every
  registered type), Hollowbell (a 2D platformer of 189 entities) to about
  25 KiB, Kestrel Stair (a 3D world of 93 entities: five skinned
  characters on graphs, five cloths and a rope, five navmesh agents with
  minds) to about 78 KB. The store takes 64 KiB a key (a platform limit,
  with 64 keys and 1 MiB per player and game), so a world goes through
  `og.platform.saveSlot`/`loadSlot`, never one `storeSet`: the slot's
  text is cut into parts of a key each (at most eight: 512 KiB), written
  in the bank of keys its last save did not use (`slot1.a0..`,
  `slot1.b0..`), and the manifest (`slot1`: the bank, the parts, the
  length, a checksum and your `meta`) last, at the revision read, so a
  save cut short leaves the last one whole and another device's save
  landing meanwhile is a `conflict`, never a mix. One request answers
  for all of it; loadSlot checks the parts against the manifest. The
  text is already deflated: a tighter encoding was measured (deflate at
  its highest level saves 0.7 %, LZMA 31 % at the cost of a second
  decoder in every build) and not taken.

  Physics carries a triangle-mesh, convex-hull or height-field collider
  by its geometry's handle and scale, not its triangles (a 956 m
  circuit's road and walls, 6,700 triangles, were 384 KB of a save
  before; they are now a few bytes): load the glTF or call
  `og.physics.setMesh` before restoring, as for every asset, or the
  restore is refused naming the mesh.

  A restore asked for in a system or a command applies when the tick
  ends; in `start`, at once. A save taken in a system is the middle of
  its tick: restored, the next step completes that tick (the systems
  after the saver and the physics step), so a replay from it (the
  countdown's world restored, the taped inputs fed again) runs the saved
  race tick for tick. The command channel's scheduled steps keep their
  remaining delays across a restore (og.Time takes the saved tick), so a
  playtest that taps Continue onto a save of a later tick, or an earlier
  one, runs the rest of its script and ends at its last step or `quit`.
  Take it as the system's last write of the tick:
  what the system does after the call is not in it. A saved type whose fields changed since (a
  release added one) is refused naming the type and fields; pass
  `{ migrate: true }` to map it by field name (new fields take their
  defaults), and keep a saved type's fields stable between releases where
  you can. Keep progress in components and resources, never in module
  variables (they are not in the save). Everything in the world rolls
  back with it, request numbers and the session's own state (a best score
  read from the store, the settings, an open menu) included, so reset
  them after it: the tick after a restore delivers `og.WorldRestored`
  (`from`, the tick the world was at; `tick`, the saved one). A system
  that reads it (`reads: [og.WorldRestored]`, `world.events(og.WorldRestored)`)
  forgets the request numbers, asks the store again and closes what the
  player had open:

  ```ts
  if (world.events(og.WorldRestored).length > 0) {
    g.progressRequest = 0;                           // answered long ago, in another session
    const asked = og.platform.storeGet(world, "settings");
    g.settingsRequest = asked.ok ? asked.value : 0;  // the language and volumes live on in the session
  }
  ```

  It is sent as that tick starts, so the world between the restore and
  that tick hashes as the save did; after a save taken mid-tick it comes
  with the first whole tick after the completing step. Asset
  handles are saved as numbers: load in `start`, in the same order every
  launch.
- **A record** (`storeSet(world, { key: "save", value: { chimes, glimmers,
  checkpoint } })`): a few hundred bytes of what progress means, rebuilt
  into a world on Continue. Smaller, readable on a title screen, and it
  survives any change to the game's types; right for checkpoint games
  (examples/hollowbell's `src/save.ts`).

Test either across a relaunch with a playtest's `store: "keep"` and
`then` (og-platform-sdk §4). A watch replay (the start's snapshot and a
tape of inputs) and a ghost (a tape of poses) are og-core §2's
"Recording, replaying and ghosts".

## 9. Checking a run: describe, expect, trace, write-back, budgets

These are the checks to use before opening frames. All of them run from
`og run --commands`, a playtest's steps or the live probe (`og probe`).

- **`describe[:path]`** (probe `describe`) writes the current frame as
  JSON. `entities` holds every visible entity, nearest first (`order`):
  `id` (the probe's id), `name`, `archetype`, `labels`, `box` {x, y, width,
  height} of its visible pixels and `point` (a pixel `pick` answers it at),
  both in CSS pixels of the play surface, `drawn` (its box drawn alone),
  `depth`, `coverage` and `occlusion` (0–1, the share of it others hide).
  `ui` lists the UI elements with `box`, `kind` and `text`, and `camera`
  the camera. The boxes come from an entity-id pass over the
  extracted frame on the CPU, so a run without a renderer describes the
  same frame. `pick` at an entity's `point` answers that entity.
- **`expect:<op>`** asserts at its tick. A failure is logged with the tick
  and the observed value, `og run --json` lists it under `expectations`,
  and the run exits 1:

  ```text
  expect:entity:crate:within:380,360,280,240    its described box inside the screen box
  expect:entity:crate:within:-3,0.4,-1,1,0.6,2  its position inside the world box (x0,y0,z0,x1,y1,z1)
  expect:event:og.CollisionStarted[:2]          at least one (or exactly 2) since the script began
  expect:ui:text:=Started                       a UI element shows the text (= exact, else a substring)
  expect:region:560,16,64,24:color:#9da5b1:24   the region's mean colour within 24 per channel
  expect:frame:references/start.png:rmse:<=6    the frame against a reference PNG
  expect:state:score>=3                         a fact of GameDesc.facts by dotted path (== != < <= > >=)
  ```

  A playtest may be expect steps alone: `templates/game/tests/smoke.og.ts`
  is. Each expectation counts as a check. Without a renderer a frame or
  region expectation is unchecked, never failed. Take screen boxes from a
  `describe` of a good run, with a margin.
- **`og run --trace out.json`** writes the run in the Chrome trace format;
  open it in Perfetto (ui.perfetto.dev). Every span lies where it ran.
  The engine thread holds each tick with its systems, render each frame
  with its render passes (CPU encoding), gpu the same passes as the GPU
  ran them (where `og.render.capabilities(world).gpuTimestamps`), load
  each cooked file read, cook the cook and each output on the cook thread
  that made it, and `lane N` the jobs og-lanes ran on pool thread N
  (physics, cull, anim, extract). The probe's `state().timings.systems`,
  `.passes` and `.gpuPasses` give the last tick's and frame's numbers
  live.
- **`save_scene[:path][:spawned]`** (probe `saveScene(write, spawned)`)
  writes the scenes back after `teleport`, `set` or the Tune panel moved
  things: the boot scene and every scene `world.instantiate` made, each
  to its own `.ogt`. Only the lines whose values changed are rewritten;
  comments and order stay, and a tuned field the text lacks gets a line.
  Resources are scene text (`resource Run { chimed true }` at the top
  level of a scene); a tuned resource no scene sets gets a block in the
  boot scene. `spawned` also writes the entities your code spawned under
  `# ── spawned at run time ──` at the end of the boot scene; if you keep
  them, guard their spawn (`if (world.find("bolt") === null)`) so the next
  run does not make them twice. With a path, the texts go under `--out`
  to diff first; without one, they replace `assets/<scene>.ogt`. `set`
  names engine components by their registered name
  (`set:marker.og.Transform.scale=0.5,0.5,0.5`).
- **`og cook --report`** writes `og-out/cook-report.json`: per asset its
  triangles per LOD, vertices, textures (size, format), bones, clips,
  bytes cooked and flags, and for every other asset its figures under its
  kind (`sound` seconds/channels/sampleRate/chunks, `font`
  glyphs/kerningPairs/pages, `scene` entities/instances/components,
  `navmesh` tiles/polygons/vertices, `timeline` tracks/seconds, `strings`
  locale/keys, `style`, `behaviour`, `tileset`, `tilemap`, `rig`,
  `rig2d`, `flipbook`, `shader`, `post`, `emitter`, `look`, `palette`,
  `lut`, `environment`, `video`, `terrain`, `streamPack`). **`og doctor
  --project . [--json]`** checks it against `budgets { }` in project.ogt
  and exits 1 on an over-budget asset, naming the asset, the limit, the
  value and the fix. The keys are `triangles` 100000, `vertices` 65536,
  `textureSize` 2048, `bones` 128, `clips` 64, `assetBytes` 8 MiB,
  `totalBytes` 64 MiB, `soundSeconds` 600, `videoSeconds` 300,
  `fontGlyphs` 8192, `navPolygons` 65536, `sceneEntities` 10000 and
  `emitterParticles` 16384, each a default a project may move.
  `bvhShare` 30 is a warning, not a limit: a model whose cooked raycast
  BVH is over 30 % of its blob (and at least 16 KiB) is named by `og cook
  --report` and `og doctor`; write `model { raycast false }` in its .ogt
  when nothing raycasts against it.

## 10. Where to look things up

`og api find <words>` and `og api show <name>` (og-core §7) answer one API
question at a time, from the API of the engine installed.

`og api show <module>` (`og api show input`) gives a module's overview and
the skill that teaches it (as in §1); `og api show <exact name>` gives a
type or function with the IDL text, defaults, ranges and units of every
field.

`guides/three-to-og.md` (beside this skill) maps three.js's classes and
idioms to the engine's: the scene graph to entities and og.Parent,
Object3D to og.Transform, the geometry classes to og-model §8's generators, the
materials to StandardMaterialDesc, the loaders to og.load, the render
loop to systems on the tick, Raycaster to `og.render.raycast`, and the
gaps by name. `og migrate three <file.ts>` rewrites a three.js file's
mechanical part with those mappings and marks every line it cannot,
citing the guide's section.
