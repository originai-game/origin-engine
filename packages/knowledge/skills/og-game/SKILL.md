---
name: og-game
description: Building a game on Origin Engine — the project layout, components and systems on the ECS, the game entry, scenes as .ogt text, assets cooked from assets/ to cooked/ (glTF and MagicaVoxel models), the og.load loaders, streaming groups, sprites, texture arrays and drop shadows, skeletal and stepped animation, effects and physical materials, looks (the frame's art direction as data: og.Stylise, og.MaterialStyle, presets, the art contract), tone mapping, exposure, environment maps, shadows and antialiasing, fog, sky, day and night, weather and water, terrain, material shaders in WGSL, physics beyond rigid bodies (joints, character and platformer controllers, vehicles, ropes, cloth, 2D and 3D fluids, ragdolls) and its queries, footage and og render's videos, and the performance budget; the entry point, with the map of the other nine skills and the modules each teaches. Use first, and when creating, structuring or extending a TypeScript Origin Engine game (a project with a project.ogt; not a Godot project.godot).
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
properties (§8); flipbook frame events driving footsteps, strikes and a
bell; texture arrays as world sprites, screen sprites and emitter layers;
streaming groups of `.vox` models and textures loading as the camera
neared them (§6); a look applied before `start` and a second one at a tick
for the results (§15); `og.physics.overlap` on a layer and with sensors;
a character controller shoving a pile of bodies (§13); a skinned glTF on
twos (§9). In The Kestrel Stair: one clip library retargeted at cook time
onto a Mixamo-named, A-posed character, and at run time
(`og.anim.retarget`) onto an Unreal-named one (its bones detected) and
onto one whose bones only a `type rig` file names, all three playing
through blend trees the first time, the clips' `hit` markers coming
through both retargets (§9); a slam lunging about a metre by root motion,
`Skeleton.rootMotion` planar for its length; a navmesh baked from the
level's glTF (og-ai §1); the 3D character controller over triangle-mesh
slopes, steps and ledges, refusing a scree too steep (§13); a cape, four
banners in a gusting wind and a chain with a lamp at its last link (§13);
`world.snapshot()` and `world.restore()` across a relaunch, bringing back
93 entities with their cloth, paths and minds (§17). `og cook` took their
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
   (§5, §8, §10). Rerun it after a change; never hand-edit its outputs.
4. **Cook** (`og cook`, og-build §1): read every refusal, it names the line
   and the fix.
5. **In 3D, the navmesh before the minds** (og-ai §1): bake it from the
   level (its glTF's `model` line), load it in `start` and check
   `og.nav.nearest` at every spawn and route point; only then write the
   behaviours.
6. **Run with commands** (og-build §1, §4): `og run . --headless
   --commands "...;screenshot:x.png;...;quit"`, then open each frame and
   say what it shows. Do this after every change that shows on screen.
   Add `describe:frame.json` beside a screenshot to get the frame as data:
   the visible entities nearest first, each with its screen box, pick point,
   depth and the fraction hidden; the UI elements with rects and text; the
   camera (§21).
7. **Playtests with reference frames** (og-build §3): `tests/*.og.ts`
   asserting on facts, `get:` answers and dumps, or written as `expect:`
   steps alone, which `og run` itself checks (§21), one per feature, and a
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
  look is lost: write it in the look, or in `start` after it (§15).
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

| Skill | Read it for | Its modules (`reference/<module>.md`) |
|---|---|---|
| **og-core** | the rules every line obeys: TypeScript only, the fixed tick, Results, archetype views, reflection | math, reflect, world, system, rng |
| **og-game** (this one) | the project, scenes, assets and loading, input, 2D, animation, effects, atmosphere, terrain, shaders, physics, performance, looks, light and shadows | game, transform, input, load, stream, render, physics, atmosphere, anim, tilemap, terrain |
| **og-build** | the loop: cook, run with a script, read frames and dumps, hot reload, playtests, `og polish` (§5 there: the one list of what it scores), shipping | commands, probe |
| **og-feel** | how much: hit stop, shake, squash, easing, floating text, the camera, frame rates, sorting, parallax, controller tunables | time, tween, camera, systems |
| **og-ui** | screens and HUDs: templates, widgets, focus, phones, UI styles and motion | ui |
| **og-taste** | the look: a style from the premise, composition, colour, type, motion, feedback, atmosphere by intent, the done checklist | — (it uses ui and atmosphere) |
| **og-model** | a 3D model built as code from a reference image: the detail inventory, eight gated passes, the generators' recipes (§18 has their fields), budgets, the review frames and `og compare` | — (it uses render and shapes) |
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

Each module's page names its skill, and `reference/index.md` groups the
modules the same way. When two skills seem to touch the same thing, the one
in this table owns it and the other points there.

## 2. The project

| Path | What it is |
|---|---|
| `project.ogt` | The manifest: `name`, `id`, the pinned `engine` version, `entry`, `profile` (`2d` or `3d`), `tick`, `seed`, `scenes`, the first `scene`, the game's `look`, the `screens` `og style check` captures (og-build §8), `publish` settings |
| `src/main.ts` | The game entry: its default export is `og.game({...})` |
| `assets/` | Source assets: glTF and `.vox` models, PNG/JPEG textures, WAV sounds, TTF/OTF fonts, `.ogt` scenes and looks, and `<file>.ogt` import settings |
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
(§15), before any asset cooks, so a game with a look names it here.

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

`reference/game.md` is `og.game` and every `GameDesc` field;
`reference/transform.md` is `og.Transform`, parents and the matrix helpers.

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
  (ETC1S for colour, UASTC for normal and linear maps), WAV into QOA, fonts
  into MSDF atlases, scenes into component tables. Import settings live next
  to the source as `<file>.ogt` (`texture { kind normal }`,
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
  scene's `model "models/hut.vox"`. The `voxel-worlds` look (§15) suits
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
  the frames' models: play them as skinned clips (§9). `voxFps 8` sets the
  rate (default 10), `voxClip walk 0 3` names frames 0 to 3 (repeatable;
  without it one clip `frames` holds the playback range set in the editor,
  else every frame; a node marked to loop repeats its keys over the file's
  frames). Each frame is centred on
  its own pivot, as in the editor: give the frames one height (or pad them)
  for a character whose feet stay on the ground.
- An `.ogt` of `type model` is a model written as text: og-geom parts by
  their `og.render` names, placed, with materials and pivots (§18).
- Reusable production assets come through the platform's asset catalog into
  `assets/`; record each file's licence next to it.

## 6. Loading

`og.load.*` returns a handle at once and completes at a tick boundary; under
`og run` and in replays every load completes before the next tick, so the
tick it lands on is deterministic. Paths are asset ids (the path under
`assets/`). Check `og.load.status` (or the `AssetReady` / `AssetFailed`
events) before using an asset, and spawn a loaded scene or model with
`world.instantiate(asset, { name, position })`. The scene in
`GameDesc.scene` is spawned for you when the world starts. `reference/load.md`
lists every loader and what each returns.

**Streaming groups** (`reference/stream.md`) keep a big world's assets off
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
(og-build §1). The fields and methods are `reference/input.md`.

A press happens somewhere inside a tick, and the tick reads it at its
start: `og.KeyDown`, `og.KeyUp`, `og.PointerDown` and `og.PointerUp` carry
`at`, when it happened in ticks from the start of the tick that reads it
(-1 to 1; from the browser event's time in a page, from the step's time
in a command script; 0 for the edges a tick's end raises and for gamepad
presses, which are polled). Timing that matters to the millisecond (a
rhythm game's judgement) uses `time.elapsed + e.at * time.dt`; everything
else ignores it. The stamp is world state, so a replay reproduces it.

## 8. 2D: sprites, flipbooks, tilemaps

- **Sprites** are components: an entity with `og.Transform` and `og.Sprite`
  (`width`, `height`, `anchor`, `color`, `opacity`, `layer`, `sortY`,
  `flipX`, `flash`, `shadow`, nine-slice `sliceX`/`sliceY`) is drawn every
  frame at its transform, its scale multiplying the size (so squash and
  stretch is a tween of `og.Transform.scale`). `og.render.sprite` stays for
  one-tick, immediate drawing (HUDs, debug). Sprites and og.ui's frames
  are filtered anisotropically where the device can
  (`og.render.capabilities(world).spriteAnisotropy`: 16, or 1 on a WebGL2
  without `EXT_texture_filter_anisotropic` and on a software rasterizer), so a sprite or a nine-slice
  squeezed along one axis keeps the other axis's detail (a button's
  outline stays sharp); at 1 it blurs, and og.ui bounds a frame's middle
  at twice its art for that reason.
- **Pixel sprites in a 3D set** stand on the ground with `og.Sprite.dropShadow`
  (or `SpriteDesc.dropShadow`): 0.5 casts the sprite's shape, darkened and
  flattened onto the ground plane (y 0) along the sun, the first
  `og.DirectionalLight`, as a 3D model's shadow would fall. It needs a
  perspective camera over the xz ground (a 2D camera sees the plane edge
  on), draws nothing with the sun below the horizon, and set geometry in
  front hides it. `shadow` is the soft blob under the feet instead; the
  `pixel-3d-hybrid` look (§15) wants both.

  ```ts
  world.insert(hero, og.Sprite, { texture: tex, width: 1, height: 1.5, anchor: { x: 0.5, y: 0 }, dropShadow: 0.5, shadow: 0.3 });
  ```
- **Billboards** (three.js `Sprite`): `og.Sprite.facing` (or
  `SpriteDesc.facing`) turns a world sprite to the camera that draws it.
  `"cameraY"` stands it upright and turns it about y (trees, characters,
  props a looking-down camera must not tip back); `"camera"` lays it in
  the view plane (smoke, glints, signs); `"none"`, the default, keeps the
  xy plane. Every camera turns it to itself (og.CameraView, og.Mirror, reflection
  probes), the turn is render-only (never read by systems or snapshots),
  faced sprites draw back to front with the transparent meshes (a tree
  behind a glass pane shows through it, one in front draws over it) and
  are culled off screen, and a faced sprite's `dropShadow` is its whole
  shape from any angle. It is still a sprite: flash, glow, blend, 2D
  lighting, arrays, flipbooks and `toneMapped` all apply. Its layer no
  longer orders it against other sprites; give it `sortY` to keep the
  sprite order instead (then it draws after the transparent meshes). World
  text takes the same `facing` (`og.render.text`, `og.render.floatingText`):
  a name over a character, a damage number over a 3D fight, turned as one
  panel about its position (centred on it with `align: "center"`; a
  floating text always is).

  ```ts
  world.insert(tree, og.Sprite, { texture: treeTex, width: 2, height: 2.6, anchor: { x: 0.5, y: 0 }, facing: "cameraY", dropShadow: 0.6 });
  ```
- **World UI that must not change with the light** (speech bubbles, name
  tags, markers) takes `og.Sprite.toneMapped: false` (or
  `SpriteDesc.toneMapped`), three.js's `Material.toneMapped`: still placed
  and depth tested in the world, but shown in its colours as given,
  outside auto exposure, bloom, fog and the tone curve, so a cream bubble
  stays cream at dusk instead of blowing out to white. Alpha-blended
  sprites only.
- **Sprite sheets** are `.ogt` files of `type flipbook` next to the PNG
  (grid or named frames, or an Aseprite / TexturePacker JSON `atlas`; clips
  with `fps`, `mode loop | pingPong | once` and `event <frame> "name"`;
  every line is the grammar at the end of this section).
  `og.load.flipbook(world, "sprites/hero.ogt")` answers the sheet and its
  clips; `og.Flipbook { sheet, clip: og.anim.clip(info, "walk") }` on the
  sprite's entity plays it, and `og.FlipbookEvent` reports the frames named
  by `event` (footsteps, the frame a hit lands). Give pixel art
  `texture { kind pixel }` import settings (lossless, nearest).
- **Texture arrays** batch frames: `og.load.textureArray` (a `type
  texture-array` of same-sized images) and `og.Sprite { array, arrayLayer }`
  (or `og.render.sprite`'s `array`, world or `screen`) draw every sprite of
  one array in one draw whichever layer each shows, glow and blend
  included. A flipbook with `array true` cooks its frames into an array
  (frame n is layer n, one size); a 2D emitter's `array "<id>" random` (or
  `life <cycles>`, `fps <n>`) draws its particles from the layers.

  ```text
  type texture-array
  version 1
  ---
  kind pixel                 # color (default), normal, linear or pixel
  layer "tiles/grass.png"    # 1 to 256 layers, all one size
  layer "tiles/dirt.png"
  ```

  ```ts
  const tiles = must(og.load.textureArray(world, "tiles/ground.ogt"));   // TextureArrayInfo: array, width, height, layers, bytes
  world.insert(cell, og.Sprite, { array: tiles.array, arrayLayer: 1, width: 1, height: 1 });
  ```

  `og.Sprite.arrayLayer` past the last layer draws the last, while
  `SpriteDesc.arrayLayer` past it is refused. An array sheet's
  `FlipbookInfo.array` is its array (its `texture` is 0). Emitters take
  arrays in sprite mode only (`render sprite <layer>`), with `round true` to
  fade each particle to a round edge; materials and terrain layers do not
  sample arrays.
- **Tilemaps**: a tileset `.ogt` (`type tileset`: grid, `solid` tiles,
  `collide` rects, animated tiles, `terrain` auto-tiling rules) and a map:
  a Tiled `.tmj`, an LDtk `.ldtk` or a hand-written `.ogt` of `type tilemap`
  (layers of rows, objects). `og.load.tilemap` answers its layers and
  objects; an entity with `og.Tilemap { map }` draws it from its position
  (top-left corner) and gives the collision layers static colliders.
  `og.tilemap.setTile` edits a cell (terrain layers re-tile their
  neighbours, colliders follow); `getTile` answers the tile there (on a
  terrain layer the one auto-tiling chose, -1 when empty), `cellAt` and
  `cellCenter` convert positions. Every collision layer's colliders sit
  on `og.Tilemap.colliderLayers` unless the layer names its own
  `colliderLayers` and `colliderMask` (grammar below): a layer of grates
  on the bit of the platformer's `oneWayLayers` is one-way while the
  walls on another layer stay solid. Layers scroll by their `parallax`;
  `repeat x` tiles a background (`reference/tilemap.md`). Every line of a
  tileset and a map, and what the cook reads of a Tiled map, is the
  grammar at the end of this section.

How many frames, how fast, which layer and how much parallax is og-feel
§9–§11. `templates/archetypes/topdown` (a Zelda-like dungeon: a Tiled map
with auto-tiled walls and collision, a four-direction flipbook hero on a
top-down character controller, an enemy with a hurt flash and knockback,
pickups with floating text, a door to a second room the camera scrolls to),
`templates/archetypes/runner` and `templates/archetypes/match3` use all of
this; `templates/archetypes/roguelike` carves a tilemap at run time from
og.Rng and keeps its fog of war on a layer, and
`templates/archetypes/tower-defense` writes placements to a map's layer.

### Flipbooks: the grammar

A **flipbook** is an `.ogt` of `type flipbook` beside its PNG (give the
PNG `texture { kind pixel }` import settings for pixel art). Its frames
are the grid's (row-major from the top left, named `0`, `1`, ... by
their index), then the `frame` lines', then an atlas's; its clips are
numbered in file order from 0, and that index is what
`og.anim.clip(info, "walk")` answers and what `og.Flipbook.clip` holds,
so a system that compares `og.Flipbook.clip` with a constant relies on
that order (look the indices up with `og.anim.clip` in `start` instead).
Every line, with its default:

```text
type flipbook
version 1
---
texture "hero.png"        # the sheet (required): beside the .ogt when that file exists, else an asset id
size 256 80               # the texture in pixels: the cook fills it in from the PNG
filter nearest            # nearest (default, pixel art) | linear
array false               # true: the frames become the layers of a texture array, one draw for every sprite (one size each)
pivot 0.5 0               # the point of each frame at the entity's position, from its bottom left (default 0.5 0.5): 0.5 0 stands on the feet
margin 0                  # grid only: pixels round the grid (default 0)
spacing 2                 # grid only: pixels between cells (default 0)
grid 32 40 8 2            # <cell width px> <cell height px> [<columns> <rows>]: frames 0.., row-major
frame slash 0 80 48 40 0.25 0   # a named frame: <name> x y w h [pivot x y], x y from the texture's top left
atlas "hero.json"         # an Aseprite or TexturePacker JSON: its frames (and Aseprite's tags as clips)
clip idle {               # clip 0
  frames 14 15            # frame indices or names, in order (required)
  fps 2                   # frames a second (default 12) ...
}
clip run {                # clip 1
  frames 0 1 2 3 4 5 6 7
  durations 0.07 0.07 0.1 0.07 0.07 0.07 0.1 0.07   # ... or seconds per frame, one per frame
  mode loop               # loop (default) | pingPong (or pingpong) | once (holds the last frame, sets og.Flipbook.finished)
  event 2 "step"          # entering clip frame 2 sends og.FlipbookEvent "step" (at most 63 bytes)
  event 6 "step"
}
```

An **atlas** names a JSON exported beside the PNG: Aseprite's (Export
Sprite Sheet, JSON hash or array: frames in the natural order of their
names with their own durations, and each `frameTags` entry a clip, its
direction `forward`, `reverse` or `pingpong`) or TexturePacker's (JSON
hash or array, its pivots taken; rotated frames are refused). The tags'
clips come first, in the atlas's order, then the sheet's own `clip`
blocks; a clip the sheet declares wins over a tag of the same name (to
give it `event` lines or another `mode`). `templates/archetypes/runner`'s
`assets/sprites/courier.ogt` is a grid sheet with five clips.

### Tilesets, maps and Tiled: the grammar

A **tileset** is an `.ogt` of `type tileset` beside its atlas PNG (give the
PNG `texture { kind pixel }` import settings). Tile ids count from 0, left
to right and top to bottom of the grid. Every line, with its default:

```text
type tileset
version 1
---
texture "tiles/dungeon.png"   # the atlas, an asset id (required)
grid 16 16 8 8                # <tile width px> <tile height px> <columns> <rows> (required)
margin 0                      # pixels round the grid (default 0)
spacing 0                     # pixels between tiles (default 0)
filter nearest                # nearest (default) | linear
solid 0 1 2 3                 # tiles that collide over the whole cell (any number of lines)
tile 50 {
  collide 0 8 16 8            # a collision rectangle: x y w h in pixels from the tile's top-left, y down
}
tile 60 {
  solid                       # the whole cell (as a `solid` line)
}
tile 51 {
  frames 51 52 53 52          # an animated tile: these tiles in turn
  fps 7                       # its rate, or `durations 0.1 0.1 0.2 0.1` (seconds, one per frame)
}
terrain wall {                # an auto-tiling terrain, named by a map layer
  mode blob47                 # blob47 (eight neighbours, 47 shapes) | bitmask16 (four edges, 16 shapes)
  rule 0 0                    # <neighbour mask> <tile>: one line per shape drawn
  default 46                  # the tile for a mask no rule names (required)
  edge filled                 # filled (default): cells off the map count as filled | empty
}
```

The neighbour mask sums the neighbours filled with the same terrain:
bitmask16 counts N 1, E 2, S 4, W 8; blob47 counts N 1, NE 2, E 4, SE 8,
S 16, SW 32, W 64, NW 128, a corner only when both its edges are set, so
exactly 47 masks occur and the cook refuses a `rule` for any other with
its line. The collision of a tile counts only on a layer marked
`collision`. `templates/archetypes/topdown/tools/make-assets.py` draws the
47 wall shapes and writes their rules (`BLOB`, the sorted reduced masks,
tile id = index); its `assets/tiles/dungeon.ogt`, the rules shortened:

```text
type tileset
version 1
---
# tiles 0-46 are the wall terrain's 47 blob shapes, 47-49 floor, 50 moss, 51-53 the torch, 54-55 the door
texture "tiles/dungeon.png"
grid 16 16 8 8
filter nearest
solid 0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30 31 32 33 34 35 36 37 38 39 40 41 42 43 44 45 46
tile 51 {
  frames 51 52 53 52
  fps 7
}
terrain wall {
  mode blob47
  rule 0 0
  rule 1 1
  rule 4 2
  # ... one rule per reduced mask, in order: 5 7 16 17 20 21 23 28 29 31 64 65 68 69 71 80 81 84 85 87
  # 92 93 95 112 113 116 117 119 124 125 127 193 197 199 209 213 215 221 223 241 245 247 253
  rule 255 46
  default 46
  edge filled
}
```

A **hand-written map** is an `.ogt` of `type tilemap` (the runner's and
the tower-defense archetype's `assets/maps/`):

```text
type tilemap
version 1
---
size 32 18                 # cells across and down (required)
tile 16 16                 # a tile's pixels (required)
unit 1                     # world units per tile width (default 1)
tileset "tiles/dungeon.ogt"   # one line per tileset, indexed from 0 (at least one)
layer walls {
  tileset 0                # which tileset (default 0)
  z 5                      # sprite layer offset from og.Tilemap.layer (default: the layer's index)
  parallax 0.5 1           # scroll factors (default 1 1; 0 stays with the camera)
  offset 0 0               # world units (default 0 0)
  opacity 1                # (default 1)
  visible true             # (default true)
  collision true           # its tiles' collision becomes static colliders (default false)
  colliderLayers 4         # those colliders' og.Collider.layers bits (default: og.Tilemap.colliderLayers)
  colliderMask 3           # the layers they collide with, og.Collider.mask bits (default: every layer)
  lit false                # the layer takes no 2D light (og.Lighting2D); default true: lit like a sprite in the lit range
  repeat x                 # x, y or both: tile the layer endlessly (a background)
  terrain wall             # a terrain layer: cells are 1 (filled) or 0, the terrain picks the tiles
  legend "#" 1             # a character and its tile id (-1 empty; on a terrain layer 1 filled)
  row "################"   # one row per map row, top first, `size` wide; "." is empty unless legended
  # or numeric rows: row 3 3 4 -1 5 (tile ids, -1 empty)
}
object door {
  kind "door"
  rect 15 8 1 2            # x y w h in cells (y down); or `at 4 6` for a point
  prop target "room2"      # a property, answered as text in TileObject.properties
}
```

**Tiled** (`.tmj`, saved as JSON): orthogonal and finite maps, layer data
as CSV or uncompressed Base64 (a compressed layer is refused with the menu
to change), horizontal and vertical flips kept (a rotated tile is
refused), group layers flattened, one tileset per layer. An external
tileset's `source` resolves against the map's folder and its extension
becomes `.ogt`: `maps/tomb.tmj` naming `../tiles/dungeon.tsx` reads
`assets/tiles/dungeon.ogt`, which holds the collision, animation and
terrains (the `.tsx` is Tiled's own and may sit beside it; the cook
refuses a missing `.ogt`, naming the path). An embedded single-image
tileset is imported with its tiles' animations, the first rectangle of
each tile's collision editor and a boolean `solid` tile property. What the
cook reads of a layer and an object:

<!-- grammar: tiled -->
| Tiled | Where | Read as |
|---|---|---|
| `autotile` | tile layer custom property (string) | the layer is a terrain layer of that terrain of its tileset; any tile in a cell marks it filled, the terrain picks the drawn tile |
| `collision` | tile layer custom property (bool) | the layer's tile collision becomes static colliders |
| `colliderLayers`, `colliderMask` | tile layer custom properties (int) | the og.Collider `layers` and `mask` bits of that layer's colliders (default: og.Tilemap.colliderLayers, every layer) |
| `lit` | tile layer custom property (bool) | false keeps the layer out of og.Lighting2D (a sky or window layer); default true |
| `z` | tile layer custom property (int) | the sprite layer offset from og.Tilemap.layer (default: the layer's index among the tile layers) |
| `repeatx`, `repeaty` | tile layer custom properties (bool), or Tiled's own layer fields | the layer repeats along x or y (`repeat` is not a name the cook reads) |
| `parallaxx`, `parallaxy` | Tiled's own layer fields (Layer Properties, Parallax Factor) | the layer's parallax |
| `offsetx`, `offsety` | Tiled's own layer and group fields | the layer's offset, a group's added to its layers' |
| `opacity`, `visible` | Tiled's own layer fields | as named |
| `type` or `class` | an object's | TileObject.kind |
| any other | an object's custom properties | TileObject.properties, every value as text |
| `solid` | an embedded tileset's tile custom property (bool) | the tile collides as a whole cell |

`og cook` warns on every custom property it does not read, naming the
property, its layer, object or tile and the names read there: a tile
layer's other than the eight names above (or one whose value has another
type: a `collision` written as the string "true"), a tile's other than
`solid`, the map's, a group's, an object layer's or an image layer's own
(none is imported), and an object's class-typed one (no text to give). A
warning fails nothing; the map cooks without the property.

Objects keep their name, size and position (a tile object is anchored at
its bottom, as Tiled draws it). Positions arrive in map-local world units,
y up from the map's top-left corner. A map's import settings
(`maps/tomb.tmj.ogt` beside `maps/tomb.tmj`, for any map format):

```text
type import
version 1
---
tilemap {
  unit 1                   # world units per tile width (default 1)
  tileset "tiles/x.ogt"    # use this tileset for every tileset of the map
  level "Level_0"          # LDtk: the level, by identifier (default the first)
  solid 1 2                # LDtk: the IntGrid values that collide (default 1)
}
```

`templates/archetypes/topdown/tools/make-assets.py` writes
`assets/maps/tomb.tmj` (a floor layer, a `walls` layer with `autotile`
`wall` and `collision` true, a `decor` layer at `z` 15 drawn over the hero,
and an object layer of spawns, coins, the door and two rooms) beside its
tileset: the pattern for a map made by a script.

## 9. Animation

Skinned glTF models animate through `og.anim` (`reference/anim.md`):
`og.load.animation(world, "models/walker.glb")` answers the rig's skeleton
and clips (`og.load.gltfInfo(world, model).animations` lists the clip
names too); on the instantiated model's entity, `og.Skeleton { asset }`
poses it and `og.AnimationPlayer { clip, speed, loop, fade }` plays a clip
(changing `clip` cross-fades over `fade`; `stepFps` holds each pose for
1/stepFps seconds: 12 is stop motion on twos, 8 on threes, for clay and
puppet looks, while events and root motion stay smooth).
`og.AnimationGraph` runs a blend
tree with layered state machines registered by `og.anim.graph` (parameters
such as speed pick the motion; its grammar ends this section), `og.IkChain`
turns a chain of bones to a target after the clips, `og.Blendshapes` weighs
morph targets, and `og.AnimationEvent` reports a clip's markers (below).
An `og.AnimationPlayer` beside a graph plays over it by its `weight`: a
strike, a bite or a fall at weight 1, back to 0 when `finished`.
**Root motion** is `og.Skeleton.rootMotion`: `planar` moves the entity's
Transform by the root joint's xz travel and yaw, `full` by all of it,
`off` (the default) leaves the hips travelling inside the model; switch it
to `planar` for a lunge's clip and back to `off` after.
**Look-at and joint limits.** `og.LookAt { skeleton, joint, target }`
on its own entity turns a joint (a head, an eye, a turret's barrel) so its
`axis` (default +Z, as three.js `lookAt`) points at `target` after the
clips and IK, its `upAxis` (+Y) rolling towards `up` (world +Y; zero
keeps the animated roll). `maxYaw` and `maxPitch` (radians from the
animated forward) hold a head from turning round, `clamped` reads back
when they did; `bones: 3` spreads the turn over neck, chest and spine;
`weight` fades it in and out. `og.JointLimit { skeleton, joint, axis,
swing, twistMin, twistMax }` (radians from the rest pose) limits what
`og.IkChain` and `og.LookAt` may turn a joint to: a knee is a hinge,
`axis` +X, `swing: 0.15`, `twistMin: 0`, `twistMax: 2.6`, so a
foot IK target never bends it backwards; a shoulder is a cone (`swing`).
The defaults limit nothing, and the clip's own pose is never clamped.

```ts
const look = must(world.spawn({ name: "hero-look" }));
must(world.insert(look, og.LookAt, { skeleton: body, joint: HEAD, bones: 2, maxYaw: 1.2, maxPitch: 0.5 }));
const knee = must(world.spawn({ name: "hero-knee-l" }));
must(world.insert(knee, og.JointLimit, { skeleton: body, joint: SHIN_L, axis: { x: 1, y: 0, z: 0 }, swing: 0.15, twistMin: 0, twistMax: 2.6 }));
// each tick, from a system that writes og.LookAt: world.set(look, og.LookAt, { target: playerPosition })
```

**Foot locking.** `og.FootLock { skeleton, foot }` on its own entity, one
per foot, keeps a foot the clips plant (within 5 mm of its lowest in the
clip, the same test retargeting uses) where it touched the ground while
root motion carries the body on and clips cross-fade, bending the leg
(two-bone IK, `og.JointLimit`s respected); when the clips lift it the
foot rejoins its animation over `release` (0.15 s). `planted` and
`locked` read back each tick; `maxDistance` (0.5 m) lets go before a leg
would stretch. It needs clips whose feet actually rest: a foot that only
grazes its lowest for a frame is held for that frame only.

**Spring bones.** `og.SpringBone { skeleton, joint, bones }` on its own
entity, one per chain (a tail, ears, a ponytail, a single jiggling node),
swings `joint` and its `bones` − 1 parents after their parents' motion:
each joint's tail is a damped spring (`frequency` 3 Hz, `damping` 0.3;
1 settles without overshoot) towards where the animation puts it, under
`gravity` (`{ x: 0, y: -9.8, z: 0 }` droops it), held at the bone's
length. `space` world (default) lags behind the entity's motion; model
ignores it (a teleport does not whip the tail). Applied last, after IK
and look-at; `lag` reads back how far the chain trails. Deterministic and
saved with the world. `tipFrequency` and `tipDamping` (1) scale the last
joint's frequency and damping, the joints between in proportion: 0.25 is
a strand soft at its tip on a stiff root.

Keep hair out of the head with `og.SpringCollider { skeleton, joint,
shape, offset, tail, radius }` on its own entity, a sphere about `offset`
or a capsule from `offset` to `tail` in the joint's frame: every tail of
that skeleton's springs is pushed out of it to its `radius` plus the
spring's `radius`, keeping its bone's length. A collider's `layers` (1)
must meet the spring's `colliders` (all) to count.

```ts
const tail = must(world.spawn({ name: "hero-tail" }));
must(world.insert(tail, og.SpringBone, { skeleton: body, joint: TAIL_TIP, bones: 3, gravity: { x: 0, y: -4, z: 0 }, tipFrequency: 0.5, radius: 0.02 }));
const hips = must(world.spawn({ name: "hero-hips-collider" }));
must(world.insert(hips, og.SpringCollider, { skeleton: body, joint: HIPS, shape: "capsule", offset: { x: -0.15, y: 0, z: 0 }, tail: { x: 0.15, y: 0, z: 0 }, radius: 0.12 }));
```

`templates/archetypes/third-person` (`src/anim.ts`: a 1D blend by speed,
a hero retargeted at cook time and sentinels at run time) is the pattern,
and for a 3D action game as a whole: a CharacterController over a level,
clips retargeted at cook and at run time, blend trees, a navmesh and
behaviour machines (og-ai).
`og.anim.clip` finds a clip by name, a model's as a flipbook's (§8):

```ts
const anim = must(og.load.animation(world, "models/slime.vox"));
const chase = og.anim.clip(anim, "chase"); // a ClipAsset; 0 when the model has none
world.insert(slime, og.Skeleton, { asset: anim.skeleton });
world.insert(slime, og.AnimationPlayer, { clip: chase, loop: true, stepFps: 12 });
```

A `.vox` with frames (§5) is played the same way: its clips (`frames`, or
the `voxClip` names) swap the frames' models on the frame boundaries.

**Markers** are a clip's named times: the frame a strike lands, a
footstep. The cook reads them from the glTF, in each animation's extras
(`animations[i].extras."og.events"`, no other key):

```json
{ "name": "attack", "extras": { "og.events": [{ "name": "hit", "time": 0.42 }] } }
```

or from the model's import settings, `event "attack" "hit" 0.42` (clip,
name, seconds; the `animation` block below). A name is 1 to 64 bytes, a
time inside the clip and an `event` line's clip one of the glTF's own, or
the cook refuses it naming the clip; a library's clips bring the markers
the library carries (in its glTF or its own import settings).
`ClipInfo.markers` lists a clip's marker names in time order (an empty
list: the glTF carried none). On the tick playback passes one, through a
player or a graph, `og.AnimationEvent { entity, clip, name, time, weight }`
fires; a system that lists `og.AnimationEvent` in its `reads` reads them
with `world.events(og.AnimationEvent)`. Markers survive retargeting, at
cook time and at run time, so a library's `hit` lands on every character.

**Retargeting** — one clip on characters of other proportions and bone
names. Keep shared clips in an animation library (a glTF with a skeleton
and clips, no mesh). At cook time, a character's import settings
(`models/hero.glb.ogt`) pull them in. A model's `animation` block, every
line with its default:

```text
type import
version 1
---
animation {
  rate 30                              # resampling rate of every clip, 1 to 120 Hz (default: the source's key rate)
  event "attack" "hit" 0.42            # a marker on a clip of this glTF's own: clip, name, seconds (repeatable; or glTF extras."og.events")
  library "anims/humanoid-clips.glb"   # its clips land in this model's #anim, retargeted (repeatable)
  rig "rigs/hero.ogt"                  # how this model's bones are named (`type rig`); default: detected from the node names
  footIk true                          # retargeting solves the legs so planted feet stay planted (default true)
  mirror "turnLeft" "turnRight"        # also write a clip mirrored to the other side, under that name (default "<clip>.mirrored"; repeatable)
  compress false                       # quantised samples and keyframe reduction (default false: f32)
  tolerance 0.0001                     # the most compression may move a sampled value (default 0.0001, at most 0.1)
}
```

A clip the model has of its own wins over a library's of the same name.

At run time, `og.anim.retarget(world, { clip, source, target })` answers a
new clip for `target` (both skeletons from `og.load.animation`). The bones
are found from the node names (Mixamo, VRM, Unity, Rigify, Unreal, Blender);
when they are not, write a `type rig` .ogt (`scheme mixamo`, or
`hips "Root_Pelvis"` per bone), load it with `og.load.rig` and pass it as
`sourceRig`/`targetRig`. `og.anim.humanoid(world, { skeleton })` says which
bones were found. The target keeps its bone lengths, its hips travel the
source's distance scaled by the leg ratio (so root motion matches its
stride), and its legs are solved so planted feet stay on the ground
(`footIk`, on by default). The fx-showcase `retarget` station is the
pattern.

Hands retarget too: the fingers are humanoid bones (VRM 1.0's
`leftThumbMetacarpal` … `leftLittleDistal`, read from Mixamo's
`LeftHandIndex1`, VRoid's `J_Bip_L_Index1`, Rigify's `DEF-f_index.01.L`,
Unreal's `index_01_l`, plain `index.01.L`; Unity's legacy `Left Thumb
Proximal` is the metacarpal), each turning from its hand as
the source's does, so a fist or a grip holds on an A-posed arm whatever
the arm does. The target's twist bones (unmapped nodes named `twist`, such
as Unreal's `lowerarm_twist_01_l`, or Rigify's `DEF-forearm.L.001`) take a
share of their segment's roll by where they sit along it: none at the
segment's head, the next bone's at its end, so a forearm does not
candy-wrap. A clip made for the rig itself that leaves its twist bones
still gets the same shares each tick (after IK); a clip that keys them
is played as keyed. A character whose rest pose does not stand on the ground (an
armature lifted off it, a crouched rest) says how high its ankles stand
in its `type rig` file, `footHeight 0.08` (or `footHeight <left>
<right>`); planted feet then land on that ground, not at the rest pose's
foot height.

**Mirroring.** `og.anim.mirror(world, { clip, skeleton })` answers a new
clip that plays `clip` on the other side (a left-handed swing, a limp on
the other leg, a turn the other way): left and right nodes are paired by
the side tokens every naming scheme writes (`LeftArm`/`RightArm`,
`thigh.L`/`thigh.R`, `J_Bip_L_`/`J_Bip_R_`, `calf_l`/`calf_r`), each
node takes its counterpart's motion reflected across the skeleton's
x = 0 plane, root motion's sideways travel and turns flip. It assumes a
symmetric rest pose; mirror a retargeted clip on the skeleton it was
retargeted for.

**Compression** shrinks a model's clips in the bundle (not in memory: they
are rebuilt at load). In the same `animation` block, `compress true`
quantises rotations to 48 bits and the rest to 16 and drops the keyframes
interpolation rebuilds within `tolerance` (default 0.0001: about 0.1 mm
and 0.01°); a channel 16 bits cannot hold within it stays exact. Turn it on
for a game with many clips; leave it off while a clip's motion is still
being judged.

### Blend trees: the graph grammar

A graph is `.ogt` text of `type animgraph` passed to `og.anim.graph(world,
{ text, clips })`; each `clip` line names a clip by `ClipInfo.name` among
`clips` (a model's `og.load.animation(...).clips`, or retargeted ones).
Motions (clips and blends of them) form a tree; layers run state machines
over them. Every line, with its default:

```text
type animgraph
version 1
---
param speed                 # a parameter: og.AnimationGraph.p0, the next p1, ... (at most 8)
param wave
param x
param y
clip idle "idle"            # clip <motion> "<clip name>"
clip walk "walk" {
  speed 1                   # duration multiplier, above 0 (default 1)
  loop true                 # false: holds its last frame (default true)
}
clip walkOther "walk" {
  mirror true               # plays the clip on the other side of the skeleton (default false): no og.anim.mirror needed
}
clip run "run"
clip waving "wave"
blend1d move speed {        # blend1d <motion> <param>: the two children around the value blend linearly, clamped at the ends
  child idle 0              # child <motion> <threshold>; any order (at least one)
  child walk 1.5
  child run 4
}
blend2d strafe x y {        # blend2d <motion> <param x> <param y>: children at points, gradient-band weights
  child walk 0 1            # child <motion> <x> <y>
  child idle 0 0
}
layer base {                # the first layer is the base: root motion and og.AnimationGraph.state come from it
  state still idle          # state <name> <motion>
  state move move {
    speed 1                 # playback rate, 0 or more (default 1)
    sync gait               # a sync group: states of one group play in step across layers (default none)
  }
  entry still               # the state it starts in (default: the first)
  transition still move {   # transition <from | any> <to>; in file order, the first that holds fires
    when speed gt 0.2       # when <param> gt | ge | lt | le | eq | ne <number>; every `when` must hold
    exit 0.9                # and the source has played this many cycles (a `when` or an `exit` is required)
    fade 0.25               # cross-fade seconds (default 0.2)
    interrupt any           # what may cut this fade: any (default) | none (it completes first) | source (only the state being left) | next (only the new state's own)
  }
  transition any still {    # any: from every state but the target
    when speed le 0.2
  }
}
layer upper {
  mode additive             # override (default): replaces the pose below | additive: adds each clip's difference from its first frame
  weight wave               # a number 0 to 1, or a parameter clamped to it (default 1)
  mask "spine" 0.5          # mask "<node>" [weight 0 to 1, default 1]: that node and every node under it (default: no mask, all 1)
  mask "arm.L"              # a deeper line wins for its branch; nodes under no line take 0
  state wave waving {
    sync gait
  }
}
```

Each state advances one phase by `dt × AnimationGraph.speed × state speed
/ duration`, the duration weighted across a blend's clips, so blended
clips stay in step. Per tick and layer the transitions are checked first,
then the phases advance, then the poses blend; a transition during a fade
replaces it unless its `interrupt` says otherwise (`none` for a
committed attack or landing, `source` to let an idle re-target a start
that has just begun). A layer's `mask` lines confine it to branches of the
skeleton (by node name, the nearest named ancestor's weight; an upper-body
layer is `mask "spine"`), so a wave or an aim plays over a walk without
freezing the legs. In a sync group the first layer (file order) whose
current state is in it leads: every later layer's state of that group
plays at the leader's phase, whatever its clip's length, and a transition
between two states of one group keeps the phase (a walk-to-run blend
keeps the feet in step). `og.anim.graph` refuses a parse error naming the line, a
clip name `clips` lacks, a parameter not declared and a motion that
contains itself. Write the parameters from a system every tick
(`world.set(body, og.AnimationGraph, { p0: speed })`); `state` and
`stateTime` read back the base layer's.

The third-person archetype's hero (`src/anim.ts`): idle, walk and run by
the speed across the ground, and the run held mid-stride, slowed, in the
air:

```text
type animgraph
version 1
---
param speed
param air
clip idle "idle"
clip walk "walk" {
  speed 1.1
}
clip run "run"
blend1d move speed {
  child idle 0
  child walk 2.2
  child run 5.6
}
layer base {
  state ground move
  state air run {
    speed 0.3
  }
  entry ground
  transition ground air {
    when air gt 0.5
    fade 0.15
  }
  transition air ground {
    when air lt 0.5
    fade 0.1
  }
}
```

```ts
const anim = must(og.load.animation(world, "models/strider.glb"));
const graph = must(og.anim.graph(world, { text: HERO_GRAPH, clips: anim.clips }));
world.insert(body, og.Skeleton, { asset: anim.skeleton });
world.insert(body, og.AnimationGraph, { asset: graph });
// every tick: world.set(body, og.AnimationGraph, { p0: groundSpeed, p1: airborne ? 1 : 0 });
```

### 2D skeletons: the rig2d grammar

A cut-out character (bones, a cape that bends, feet that stay on a slope)
is an `.ogt` of `type rig2d` beside its PNG, loaded with
`og.load.rig2d(world, "rigs/hero.ogt")` (`reference/anim.md`). Pixels,
+y up, degrees counter-clockwise, `unit` pixels per world unit; bones
parent first, slots draw in file order:

```text
type rig2d
version 1
---
texture "hero.png"
filter nearest
unit 20
bone root {
}
bone hip {
  parent root
  at 0 22
  rotation 90
  length 16
}
slot torso {
  bone hip
  attachment torso
  color 1 1 1 1
  offset 0 0 0 0
}
region torso {
  slot torso
  rect 0 0 12 20
  at 8 0
  rotation -90
  size 12 20
}
mesh cape {
  slot cape
  vertex 0 32 hip 16 9 1
  vertex 6 40 hip 8 4 0.4 cape-1 7 0 0.6
  triangles 0 1 2
}
box hurt {
  slot torso
  polygon -6 0 6 0 6 20 -6 20
}
child flame {
  slot lantern
  armature fire
  clip burn
  at 15 0
}
ik leg {
  bones thigh shin
  target foot
  bend clockwise
}
armature fire {
  bone base {
  }
  # slots, regions, meshes, boxes, ik and clips, as at the top
}
clip idle {
  duration 2
  loop true
  bone hip {
    translate 0 0 0 curve 0.42 0 0.58 1
    translate 1 0 -1.5
    skew 0 0
  }
  slot torso {
    offset 0 0 0 0 0
    offset 1 0.2 0.1 0 0
  }
  deform cape {
    key 0 0 0 0 0
    key 1 0 -3 0 -6
  }
  order 1.5 cape torso
  ik leg {
    key 0 1 clockwise
    key 1 0.5 clockwise
  }
  event 0 "inhale"
}
```

`rect` is pixels from the image's top left; a mesh `vertex` is `u v` in
pixels then, per bone, `bone x y weight` in that bone's space (weights
summing to 1), or plain `x y` in the slot's bone. A bone's or region's
`skew` turns its y axis that many degrees past its x axis. A slot draws
`texel × color + offset`. Bone keys add to the setup pose (`rotate` and
`skew` degrees, `translate` pixels, `scale` multiplies); a `rotate` key
turns the shorter way to the next, or `rotate 0 0 clockwise 1` (or
`counterclockwise n`) that way through n more whole turns; every key eases
linearly to the next, holds with `step`, or follows `curve x1 y1 x2 y2`
(a cubic Bézier from 0,0 to 1,1, x time and y value; more segments as
`x y x1 y1 x2 y2`). A `deform` key offsets every influence in order, one
`x y` pair each; `order <time> <every slot>` redraws back to front from
then (alone, the file's order); an `ik` key sets the constraint's weight
and bend. An `ik`'s `bones` are a chain of any length, root first: one
bone points, two bend at the joint, three or more (a tail, a tentacle, a
three-joint leg) are solved by FABRIK and curl to the `bend` side. After the IK, a
`transform t { bones a; target b; mix 1 1 1 1 }` pulls bones towards
another bone's world rotation, origin, scale and shear by its mix, and an
`along s { path rail; bones s1 s2; position 0 percent; spacing 0 length;
rotate tangent 0; mix 1 1 }` lays bones along a `path` attachment (a
cubic spline, `vertex x y` three per knot, drawn as nothing) while its
slot shows it. A `skin armor { swap torso torso-armor; swap hat none }`
block shows other attachments of the same slots wherever the default ones
would show; `og.Skeleton2D.skin` picks it (`rig.skins.indexOf("armor")`,
`default` is 0). A `box` draws nothing: `og.anim.skeleton2dBoxes(world, hero)`
answers its polygon (in the Transform's frame) while its slot shows it,
for hit and hurt boxes a clip switches. A `child` shows an `armature`
block (a rig of its own on the same texture) in its slot; it plays the
parent's clip by name when it has one, else its `clip`, at the parent's
time, forwards or backwards; its events arrive as og.Skeleton2DEvent with
`armature` its name (empty for the skeleton's own clip). An `armature`
block takes `skin` blocks of its own, and wears the one named as the
skin the rig wears (else its default). A DragonBones 5.5 export is imported instead of written: a rig2d
`.ogt` with `dragonbones "hero_ske.json"` and `atlas "hero_tex.json"`
(skew, colour offsets, zOrder and IK timelines, armature displays,
bounding boxes, path displays and constraints, every skin, curves and `clockwise`
turns included; Spine's files are not read, convert
them to this text).

```ts
const rig = must(og.load.rig2d(world, "rigs/hero.ogt"));
const hero = must(world.spawn({ name: "hero", position: { x, y, z: 0 } }));
must(world.insert(hero, og.Skeleton2D, { rig: rig.rig, clip: og.anim.clip(rig, "idle"), layer: 1 }));
// A foot planted on the ground: one og.IkTarget2D per constraint, moved every tick.
const foot = must(world.spawn({ name: "hero-foot" }));
must(world.insert(foot, og.IkTarget2D, { skeleton: hero, constraint: rig.constraints.indexOf("leg"), target: { x, y: groundY } }));
```

`og.Skeleton2D` plays `clip` at `speed` (`time` is written each tick;
setting it seeks), wraps when `loop`, mirrors with `flipX` and draws at
sprite `layer`, lit by `og.Lighting2D` like a sprite there. Clip events
arrive as `og.Skeleton2DEvent` (`entity`, `clip`, `name`); an
`og.IkTarget2D` writes `reached` and `error` (its `weight` multiplies the
constraint's own). The pose is a function of the component, the
Transform and the targets, so a snapshot carries only the playback
position and the boxes. `templates/game-2d`'s lamp keeper
(`assets/rigs/keeper.ogt`, `spawnKeeper`, `Walk`, `Plant`) shows all of
it. A walk is the walk clip played while a system moves the entity, with
the feet's targets stepping in time: each planted for half the cycle
(sliding back as fast as the body goes), then swung a stride ahead, so
the feet stay on the ground the IK finds.

### Cutscenes: timelines and paths

A cutscene, an intro, a scripted beat or a fly-through is a `type timeline`
file, not code (`reference/timeline.md`, `reference/path.md`;
`templates/game`'s `assets/timelines/intro.ogt` and `tests/intro.og.ts`
are the pattern). Tracks of keys at seconds: between two keys the later
key's `ease` (og.tween's names: `cubicInOut`, `quadIn`, ...) mixes them,
outside them the value is held. Every value is evaluated at the
timeline's time, never accumulated, so a seek lands exactly where playing
would (a playtest or a video can start at any second).

```text
type timeline
version 1
---
name "Intro"
duration 10
path "flyby" {
  kind catmullRom          # or bezier (p0 c0 c1 p1 ...) or linear; closed true wraps
  point -5 3.2 9
  point -1.5 2.4 7.5
  point 2 2 7
  point 5 2.8 8.5
}
track camera {
  rail "flyby"             # or position/target keys, or follow "<entity>"
  key 0 {
    at 0
    target 0 1 0
    fov 50
  }
  key 9.5 {
    at 1
    target 1.5 1 0
    fov 42
    ease cubicInOut
  }
}
track transform "trickster" {
  key 2 {
    position 2 0 0
  }
  key 5 {
    position 0.5 0 1.5
    rotation 0 90 0
    ease cubicInOut
  }
}
track animation "trickster" {     # the entity's og.Model's clips (or `model "<glb>"`)
  key 0 {
    clip "idle"
    loop true
  }
  key 2 {
    clip "walk"
    loop true
    blendIn 0.3
  }
}
track caption {
  captions "narration.srt"   # beside the timeline; or word timestamps .json
  style lg                   # a UI type step
  anchor bottom
}
track audio {
  key 1 {
    sound "audio/blip.wav"
    volume 0.6
    bus sfx
  }
}
track music {
  key 0 {
    sound "audio/theme.ogg"
    fadeIn 1
    fadeOut 2
    end 9
  }
}
track event {
  key 5.5 {
    do "spawn_wave:2"
    retroactive true
  }
}
track transition {
  key 0 {   # cut, fade, wipe, dissolve, iris, slide
    kind fade
    mode in
    duration 1
  }
  key 9 {
    kind fade
    mode out
    duration 1
  }
}
track marker {
  key 5.5 {
    name "drop"
  }
}
```

The cook checks
the sounds and models it names and imports the captions, so an `.srt`
edit recooks the timeline. Play it from code or from the command channel:

```ts
const intro = must(og.load.timeline(world, "timelines/intro.ogt"));
const e = world.spawn({ name: "intro" });
world.insert(e, og.Timeline, { asset: intro });
og.timeline.play(world, e);         // pause, seek(world, e, 4), stop
// a system reading og.TimelineMarker (name "drop") or og.TimelineEnded reacts to it
```

`og run . --commands "0:timeline:play:intro;4:screenshot;8:screenshot"`
plays `timelines/intro.ogt` on an entity named `timeline-intro`;
`timeline:seek:4`, `timeline:pause`, `timeline:resume`, `timeline:stop`
take an optional `:<name>`. While a timeline with a camera track plays
(or is paused) it owns `og.Camera`; it hands back the camera it found
when it stops or ends. Event keys run their action at the start of the
next tick; a seek skips the events it passes unless they are
`retroactive`, and markers are sent only by playing past them. A capture
at `t:` shows the world after the ticks before t: the play's first tick
is the timeline's time 0, so the frame at `4:` shows 3.983 s at 60 Hz.

Paths on their own: `og.path.create(world, { points, kind, closed,
tension })` or `og.path.find(world, "<name>")` for one a scene declares
(the same `path` block in a `type scene`); `og.path.sample(world, p, t)`
gives position, tangent and up at arc-length t 0..1, `length` and
`nearest` the rest. `og.PathFollower { path, speed, loop, align, offset }`
moves an entity along one each tick (`distance` is its own field, so a
snapshot keeps it); `og.CameraRail { path }` rides one instead of its
inline points.

## 10. Effects

Every effect is off until the game turns it on, as data:

- **Post** — the `og.PostEffects` resource: `ao`, `outline`,
  `depthOfField`, `motionBlur`, `godRays`, `glow`, `lut` (a handle from
  `og.load.lut(world, "luts/look.cube")`), the colour adjustments after it
  (`temperature`, `tint`, `brightness`, `contrast`, `balanceShadows`,
  `balanceMidtones`, `balanceHighlights`, `hue`, `saturation`,
  `vibrance`, `grayscale`, `invert`, `threshold`: a photo editor's sliders,
  0 is off; baked with the LUT into one table, so they cost no pass;
  each frame that changes one re-bakes it on the CPU, about half a
  millisecond to a few, so a fade such as `post.saturation = -t` is
  fine and a slider moved on every frame of play is not),
  `vignette`, `chromaticAberration`, `lensDistortion` (barrel above 0,
  pincushion below, `lensDistortionCenter`; fitted so no black edge
  shows, the UI not bent), `screenBlur` (`"zoom"` towards
  `screenBlurCenter` for a dash or a hit, `"spin"` round it, or
  `"directional"` along `screenBlurAngle` for a fast pan; off until
  `screenBlurStrength` is above 0, and fade that back to 0 rather than
  holding it: a frame blurred for long reads as a fault), `filmGrain`, `flash`, and a fixed shake of the
  frame through `shakeTrauma` (an earthquake that lasts; a hit's shake is
  `og.camera.shake`, which decays by itself: og-feel §4). A game with fast
  motion wants `render: { antialias: "taa" }` in `og.game`.
- **Particles** — an emitter is an `.ogt` asset of `type particles`
  (shape, rate, bursts, lifetime, speed, curves over life, texture or
  array, 3D or sprite rendering; every line is the grammar at the end of
  this section) loaded with `og.load.emitter`; an entity with `og.ParticleSystem
  { emitter, seed, startTick }` emits it at its transform. Particles are
  drawn by the renderer from the tick, deterministic and never part of the
  simulation.
- **Trails and beams** — components `og.Trail` (follows the entity) and
  `og.Beam` (between two points); `og.render.ribbon(world, desc)` draws a
  polyline this tick.
- **Decals** — `og.Decal` projects an atlas rectangle
  (`og.RenderSettings.decalAtlas`) onto what its box covers.
- **2D lighting** — `og.Lighting2D` (ambient, the lit layer range),
  `og.Light2D` and `og.Occluder2D` components, `castShadow` on sprites;
  sprites also take `flash`, `glow` and `blend`.
- **Materials** — toon, rim, dissolve, hit flash and outline are
  `og.render.standardMaterial` fields; animate them with
  `og.render.updateMaterial(world, material, { dissolve: t })`. Beside
  three.js's standard fields (metalness, roughness, emissive, `normalMap` in
  glTF's tangent frame, `alphaTest`, `transparent`, `side: "double"`,
  `unlit`), three.js `MeshPhysicalMaterial`'s fields work on every tier:
  `clearcoat` (with `clearcoatMap`, `clearcoatNormalMap`) for car paint and
  varnish, `sheen` and `sheenColor` for cloth and velvet, `transmission`
  with `thickness`, `ior` and `attenuationColor` for glass and water that
  refract what is behind them, `iridescence` for soap film and oil,
  `anisotropy` for brushed metal; a glTF's `KHR_materials_*` extensions
  load into them. A non-uniform `scale` shades correctly. Set `sheenColor`
  with `sheen` (black is no sheen); `thickness` is in the mesh's units and
  `attenuationDistance` 0 absorbs nothing. A transmissive material refracts
  only what was drawn before it (not other glass or water), and unlit and
  toon materials ignore the physical fields:

  ```ts
  const amber = og.render.standardMaterial(world, { color: { r: 1, g: 1, b: 1 }, roughness: 0.02, transmission: 1, thickness: 1, ior: 1.5, attenuationColor: { r: 0.95, g: 0.45, b: 0.08 }, attenuationDistance: 0.35 });
  ```
- **Texture repeat** — three.js's `texture.repeat` is the geometry's
  `uvRepeat` here: `og.render.planeGeometry`, `boxGeometry` and
  `sphereGeometry` scale their UVs from 0..1 to 0..`uvRepeat`, and material
  textures sample with repeat wrapping, so the texture tiles. A ground grid
  every 10 m on a 1,600 m plane (`references/metal/ground-grid.png` in
  og-tests):

  ```ts
  const ground = must(og.render.planeGeometry(world, { width: 1600, height: 1600, uvRepeat: { x: 160, y: 160 } }));
  const grid = og.render.standardMaterial(world, { map: must(og.load.texture(world, "textures/grid.png")), roughness: 0.9 });
  // turn it -pi/2 about x for a floor
  ```
- **HUD** — `og.render.sprite` / `og.render.text` with `screen: true` draw
  in CSS pixels from the top-left over the finished frame, untouched by
  post effects, with an optional `clip`. `og.ui` draws in the same
  overlay at `UI_LAYER` (1000): screen sprites and text at a layer up to
  1000 draw under every UI element, above 1000 over every one (modals and
  the screen transition too), so a HUD icon made of sprites sits on a UI
  panel at `layer: UI_LAYER + 1` while its screen shows.

`templates/fx-showcase` shows every one of these on its first five stations;
every field is in `reference/render.md`. Floating text,
`og.render.floatingText(world, { text: "-12", position, color })`, pops a
damage number or a pickup that rises and fades by itself (amounts: og-feel
§7).

### Particle emitters: the grammar

An emitter is an `.ogt` of `type particles` under `assets/` (`fx/` by
custom). Every line is optional; what is left out keeps its default (a
small fountain of soft white points). A `<min> <max>` pair is a random
range per particle, one number is both:

```text
type particles
version 1
---
maxParticles 256          # the ring: the newest overwrite the oldest (1 to 1048576)
shape cone 15 0.05        # point | sphere <radius> [surface] | cone <angle°> <radius> | box <x> <y> <z> (half extents)
spread 0                  # degrees of random deviation from the shape's direction (180: every way)
rate 40                   # particles per second; 0 for bursts only
burst 0 30                # <time s> <count> [<cycles> <interval s>]: at that time of each cycle (repeatable)
duration 1                # seconds per emission cycle
looping true              # false: one cycle, then the particles live out
lifetime 1 1.5            # seconds
speed 1.5 2.5             # units per second, along the shape's direction
size 0.08 0.12            # the quad's side in world units
sizeOverLife 0 1 1 0      # <t> <value> pairs, t 0 (birth) to 1 (death), at most 8; multiplies size (default 1)
colorOverLife {           # keys <t> <r> <g> <b> <a>, linear, at most 8 (default: white, fading over the last quarter)
  key 0 1 1 1 1
  key 1 1 1 1 0
}
color 1 1 1 1             # a tint, <r> <g> <b> [<a>]; above 1 blooms
rotation 0 0              # start rotation, degrees
angularVelocity 0 0       # degrees per second
gravity 0 -2 0            # acceleration, units per second²
drag 0                    # velocity scales by 1 / (1 + drag × dt)
noise 0 0.5 0.3           # <strength> [<frequency> <scroll>]: a moving turbulence
texture "fx/spark.png"    # an asset id; left out: a soft round dot
sheet 4 4 16 life 1       # <columns> <rows> [<frames> [off | life <cycles> | fps <n> | random]]
blend additive            # additive (fire, sparks, magic) | alpha (smoke, dust)
space world               # world: born at the emitter, then free | local: carried with it
align camera              # camera | velocity <stretch>: stretched along its motion (sparks, rain)
softDistance 0            # fade where the quad nears the scene behind it (0: off)
collision 0.4 0.2         # <bounce> <friction>: bounce off the depth buffer (3D billboards, world space)
render billboard          # billboard (3D) | sprite <layer>: 2D, drawn in the sprite batch on that layer
glow 0                    # sprite mode: strength in the 2D glow
array "fx/smoke.ogt" random   # sprite mode: draw from a texture array, [<layers>] [off | life <cycles> | fps <n> | random]
round false               # sprite mode: fade each particle to a round edge over its texture or layer
```

- Directions: `point`, `cone` and `box` emit along the entity's +y
  (`spread 180` sends a point every way), `sphere` outwards. In a 2D game
  set `render sprite <layer>`: positions stay in the xy plane, sizes and
  speeds are in world units like a sprite's, and the particles take the
  sprite batch's order and glow like any sprite.
- A one-shot effect (a hit, a pickup, landing dust) is `rate 0`,
  `looping false`, one `burst 0 <count>` and a short `duration`; fire
  and restart it by writing the tick to its `og.ParticleSystem.startTick`
  (moving the entity first). Spawn it in `start` without a `startTick`:
  the default, `PARTICLES_ARMED`, means never written, so it waits
  armed and emits nothing until its first use (write `PARTICLES_ARMED`
  back to re-arm one without firing it). A lasting one (embers, motes,
  rain) keeps the defaults `looping true` and a `rate`, and emits from
  tick 0 without a `startTick`.
- `sheet` animates a sprite sheet texture: `life 1` plays its frames once
  over each particle's life, `fps 12` at a rate, `random` one frame per
  particle. `array` does the same over a texture array's layers (§8), one
  draw for the whole emitter; a billboard emitter with an array is refused.
- `collision` bounces 3D billboards off what the depth buffer shows on the
  full tier (sparks on a floor); sprite-mode emitters skip it. It is a
  look, never gameplay: particles never touch the simulation.
- The cook refuses an unknown line, a missing texture or array and a
  value of the wrong shape, naming the line.

A hit's sparks in a 2D game (Hollowbell's `fx/hit.ogt`):

```text
type particles
version 1
---
maxParticles 24
shape point
spread 180
rate 0
burst 0 10
duration 0.1
looping false
lifetime 0.18 0.32
speed 3 6
size 0.12 0.22
sizeOverLife 0 1 1 0
colorOverLife {
  key 0 1 0.95 0.75 1
  key 1 0.95 0.55 0.2 0
}
drag 4
texture "fx/spark.png"
blend additive
render sprite 12
```

`templates/archetypes/idle/assets/fx/` (taps, stars, a milestone),
`templates/fx-showcase/assets/fx/` (fire, smoke, sparks with collision,
2D embers) and `templates/archetypes/third-person/assets/fx/` hold more.

## 11. Atmosphere, water and terrain

`og.atmosphere` is data too, and off until set (`reference/atmosphere.md`):

- **Fog** — the `og.Fog` resource: `enabled`, distance fog (`density` per
  metre from `start`, or a linear ramp to `end`), height fog
  (`heightDensity` below `height`, thinning by `falloff`), `sunColor` for
  the glow looking into the sun, `sky` for how much of the sky it covers.
- **Sky** — `og.Sky { enabled, zenith, horizon, ground, sunDirection,
  sunColor, sunSize, glow, stars }` draws the sky per pixel and lights the
  scene by it; `moonSize` (degrees, 0 none), `moonPhase` (0 new, 0.25 first
  quarter, 0.5 full, 0.75 last quarter), `moonIntensity`, `moonColor` and
  `moonDirection` (zero: opposite the sun) add a moon disc over the stars.
- **Day and night** — `og.DayNight { enabled, start, speed, tilt, azimuth,
  moonSize, moonPhase }`: the clock is `start + seconds × speed` hours from
  the tick, so it replays; it drives the first `og.DirectionalLight` (the
  sun by day, a dim moon by night, where the moon disc hangs), the sky, the
  ambient light and the fog's tint. Read the clock with
  `og.atmosphere.timeOfDay(world)` (declare `og.DayNight` among the system's
  reads) to light lamps at dusk.
- **Weather** — `og.Weather { kind: WEATHER_RAIN | WEATHER_SNOW |
  WEATHER_DUST | WEATHER_MIST, intensity, wind, radius }`: particles around
  the camera, the preset's fog (an enabled `og.Fog` wins) and dimmed light.
  With an orthographic camera the particles are 2D sprites on `layer`.
- **Water** — `og.Water` on an entity draws a surface on its xz plane
  (`size`, `shallow`/`deep` colours over `depthFade`, `foamWidth` where it
  meets geometry, `waveStrength`, `reflectivity`); with an
  `og.WaterSurface` on the same entity its waves move the surface and bodies
  splash it (`og.Splash` events) and float. In 2D, `og.WaterLine` with
  `og.Water2D` is a side-on pool drawn in the sprite order. Three more
  cost frame time and are off until set: `planarReflection` (0.25 to 1, the
  share of the screen's resolution) mirrors the scene in a still lake or a
  pool you look across, where the sky reflection alone looks fake;
  `shadows` (0 to 1) darkens it under the cascaded shadow (a pier, a cliff);
  `caustics` throws rippling light on what lies below it. With the camera
  below the surface the frame's fog becomes `underwaterColor` at
  `underwaterDensity` and the surface from below shows the sky only
  straight up (on by default; set the density to 0 to see through).

**Terrain** (`reference/terrain.md`, 3D projects only: a `2d` project's
publish build has none, §2) is a height field cooked from a
`type terrain` .ogt beside a 16-bit grey PNG (or an EXR in metres), or
generated by the cook (below):

```text
type terrain
version 1
---
heightmap "island.png"     # 16-bit grey: black and white are `height`'s ends
height -20 120             # metres
size 2048 2048             # metres along x and z, centred on the entity
control "island-control.png"  # optional RGBA: layer 0 in R, 1 in G, 2 in B, 3 in A
heightBlend 0.15           # sharpen the blends by each albedo's alpha (its height)
layer {                    # a block's '}' stands alone on its line
  albedo "textures/grass.png"
  normal "textures/grass-n.png"
  tile 6
  slope 0 30
}
layer {
  albedo "textures/rock.png"
  tile 12
  triplanar true
  slope 28 90
}
layer {
  albedo "textures/snow.png"
  altitude 90 1000
  altitudeFade 10
}
```

A layer shows where its control channel (1 without a map) times its slope
band (degrees) times its altitude band (metres above the terrain's entity)
is high; give cliffs a `triplanar` layer so the texture does not stretch.
In the game, load it in `start` and place it:

```ts
const info = must(og.terrain.load(world, "terrain/island.ogt"));
const ground = must(world.spawn({ name: "island", position: { x: 0, y: 0, z: 0 } }));
world.insert(ground, og.Terrain, { data: info.data });
world.insert(ground, og.Collider, { shape: SHAPE_HEIGHTFIELD, mesh: info.geometry });
```

**A generated terrain.** Instead of `heightmap`, a `generate` block makes
the field in the cook (never both: the cook refuses the pair and names
the lines). `height` and `size` mean what they mean for a heightmap: the
field is stretched so its lowest sample is `height`'s low and its highest
its high, and no sample leaves that range. Every line, with its default
(leave out what you do not need; erosion, thermal, terraces, rivers and
the sea are off until given):

```text
type terrain
version 1
---
height -40 160             # metres
size 1024 1024             # metres along x and z
generate {
  samples 257 257          # columns and rows (2 to 4097)
  seed 1                   # another seed, another terrain of the same kind
  noise {
    octaves 6              # 1 to 12: more is finer detail
    frequency 3            # features across the longer side
    lacunarity 2           # each octave's frequency over the last's (1 to 8)
    gain 0.5               # each octave's amplitude over the last's (0 to 1): higher is rougher
    warp 0                 # domain warp in features: 0.3 to 1 bends ridges and coasts
  }
  mask island 0.4          # land fades to `height`'s low over this share of the half size
                           # or `mask ridge 1`: sharp ridged mountains (0 to 1); default no mask
  erosion {                # hydraulic erosion: seeded drops carve gullies and fill valleys
    drops 40000            # ~0.5 to 1 per sample; more is deeper erosion and a slower cook
    lifetime 30
    radius 3               # samples each drop erodes around it
    inertia 0.05
    capacity 4
    deposition 0.3
    erosion 0.3
    evaporation 0.01
    gravity 4
  }
  thermal {                # slopes steeper than talus slump into scree
    talus 35               # degrees
    iterations 20
    rate 0.5
  }
  terraces {
    step 10                # metres between steps
    blend 0.4              # 0 hard steps, 1 smooth ramps
  }
  sea 0                    # metres above the entity: og.terrain.load's seaLevel
  shore 2                  # metres either side of the sea flattened into beaches (needs sea)
  rivers {                 # carved where the flow gathers, down to the sea or the edge
    threshold 400          # samples draining through before it is a river: lower, more rivers
    depth 2                # metres
    width 6                # metres
  }
  moisture {               # what biomes read, 0 to 1
    flow 1                 # weight of the gathered flow (wet in valleys and by rivers)
    scale 64               # samples draining through for half of it
    noise 0.5              # weight of a noise
    frequency 6
  }
  biome {                  # any number; each writes its layer's control channel
    layer 0                # required: the layer (0 to 3, one the terrain has)
    weight 1
    altitude -1000000 1000000   # metres above the entity
    altitudeFade 2
    slope 0 90             # degrees
    slopeFade 4
    moisture 0 1
    moistureFade 0.1
  }
}
layer {
  albedo "textures/sand.png"
  tile 4
}
layer {
  albedo "textures/grass.png"
  tile 6
}
layer {
  albedo "textures/rock.png"
  tile 12
  triplanar true
}
```

An island with beaches, rivers and rock on its cliffs, for that file's
three layers (a block's `{` ends its line):

```text
generate {
  seed 7
  noise {
    warp 0.5
  }
  mask island 0.45
  erosion {
    drops 40000
  }
  sea 6
  shore 3
  rivers {
    threshold 300
    depth 2.5
    width 8
  }
  biome {
    layer 0
    altitude -1000 9
  }
  biome {
    layer 1
    altitude 9 1000
    slope 0 32
  }
  biome {
    layer 2
    slope 32 90
  }
}
```

With `biome` rules the generator writes the control map (a terrain with
biomes names no `control`); a channel is the most any of its layer's
biomes gives a sample, its `weight` times its three bands. Without biomes
there is no control map and the layers' own `slope` and `altitude`
place them, as for a heightmap. The same file makes the same terrain on
every machine (the generator uses only exact arithmetic and its own
random numbers). It is a credible terrain of a kind, never a terrain from
a picture: move the seed, the mask and the erosion until it reads right.
`og cook --report` gives its height histogram, the share of land above
the sea and the rivers' length, and writes its height and control maps
as PNGs to `og-out/terrain-maps/` to look at.

Its sea is the water's level; the water is the terrain's size:

```ts
if (info.seaLevel !== null) {
  const sea = must(world.spawn({ name: "sea", position: { x: 0, y: info.seaLevel, z: 0 } }));
  world.insert(sea, og.Water, { size: info.size, segments: 128 });
}
```

(on the terrain's entity's x and z, under its parent if it has one; an
`og.WaterSurface` on the same entity makes the waves move and bodies float).

It is drawn around the camera at finer detail nearby (casting and
receiving shadows, under the fog), placed by its entity's world transform
(turn or scale the entity and the terrain turns and scales with it), and
`og.CharacterController` walks on it up to `maxSlope`.
`og.terrain.sample(world, { data, origin, quaternion, scale, x, z })`
(the entity's position, rotation and scale; the last two may be left out
for an unturned, unscaled terrain) gives the height, normal and slope
anywhere (to place trees, spawn on the ground, or check a slope); `og.terrain.edit` raises, lowers, flattens,
smooths or paints under a round brush (craters, building pads, a paint
tool) and `og.terrain.layer` / `setLayer` retune a layer live. Edits are
world state: a snapshot keeps them and restoring it brings the ground
back as it was then. Grass, flowers and stones are an `og.TerrainScatter` on the
terrain's entity: `{ data, geometry, material, layer, density, distance }`
grows copies of the mesh where that layer shows, within `distance` of the
camera (thinning out over its outer `fade`, never shrinking), each fixed to its spot
as the camera moves. Keep `density × distance²` modest: 4 a square metre
within 60 m is about 36,000 copies. The resource `og.Wind` sways them:
`strength` (0 still; 0.2 leans a 1 m stem's tip 20 cm downwind),
`direction`, `speed` (how fast gusts cross the field) and `gust` (0 a
steady lean, 1 from upright to twice it); set `sway: 0` on a stone
scatter, and model a copy with its root at the mesh's origin, since each
vertex moves by its height above it. It is drawn from the tick, so it
needs no system and costs nothing in the simulation. A navmesh .ogt's `terrain "terrain/island.ogt" { step 4 }` bakes
the walkable slopes for og-ai. Lower `lodRing` on og.Terrain to trade
detail for frame time.
Its tiles stream: only the tiles the camera needs are on the GPU (16 MiB a
terrain by default), decoded off the frame, the coarser level standing in
until each arrives, and nothing a tick reads changes with it. In a probe
script `terrain_stream:resident` decodes every wanted tile each frame (for
captures to compare), `terrain_stream:stream:<MiB>` streams within that
budget, and `dump_stream` gives the terrain's tiles.

Which look for which mood is og-taste §8.

## 12. Material shaders

A material can run your own WGSL in four hooks of the lit shader. Write the
hooks in a `.wgsl` file and declare it in an `.ogt` of `type shader`:

```text
type shader
version 1
---
source "shaders/dissolve.wgsl"
param amount float 0
param edgeColor color 4 1.6 0.4
```

```wgsl
// Any of the four; the others stay the identity.
fn vertex_offset(h: Hook, p: Params) -> vec3<f32>                  // world-space offset
fn surface_color(h: Hook, p: Params, c: vec4<f32>) -> vec4<f32>    // base colour and alpha
fn surface_normal(h: Hook, p: Params, n: vec3<f32>) -> vec3<f32>   // world normal, unit length
fn surface_emissive(h: Hook, p: Params, e: vec3<f32>) -> vec3<f32> // emitted light
// h: world, local (object-space) position, normal, uv, color, time (s).
// p: the declared parameters by name (p.amount). value_noise3(p, seed) is
// the engine's 3D value noise (0..1).
```

The cook compiles every shader for every back end (WebGPU, Metal, Vulkan,
and GLSL ES for WebGL2) and refuses one that does not compile with naga's
message, its lines counted in your hooks. In the game:
`og.render.shaderMaterial(world, must(og.load.shader(world,
"shaders/dissolve.ogt")), { color, alphaTest: 0.5 })` makes the material,
`og.render.setShaderParam(world, material, "amount", [0.4])` sets a
parameter, and `og.render.setMaterialShader(world, material, shader)` gives
an existing material (a loaded model's, from `GltfInfo.materials`) the
hooks. Alpha below `alphaTest` discards the fragment (a dissolve). Skinned
meshes run the hooks too, and shadows and the depth prepass see the mesh
where `vertex_offset` moved it. `templates/fx-showcase/assets/shaders/` has a
dissolve, wind-swayed grass and a ripple on the walking walker.

## 13. Physics

`og.RigidBody` and `og.Collider` on an entity make it a body in the
profile's simulation (rapier3d in 3D, rapier2d in 2D); contacts and sensors
arrive as events, and `og.physics` answers queries. `og.Velocity` is
optional: the step adds it to a dynamic body that has none, and a kinematic
body (`BODY_KINEMATIC`) moves by it each tick, its Transform written back:
a lift is a kinematic body with a Velocity whose system only turns the
velocity round at the ends, and a PlatformerController standing on it
rides along. A kinematic body without one stays where its Transform puts
it (move that by hand), and writing a moving one's Transform teleports it
there. `world.set(e, og.Velocity, ...)` refuses on
an entity without one (a body in `start`, before its first step):
`world.insert` it there, which adds and sets it.

`og.CollisionStarted` is read on the tick after the physics step, when
the bodies' `og.Velocity` already holds the solver's answer (they may
already be parting), so never weigh a hit from og.Velocity. The event
carries the hit itself, taken in the step that made the contact. `a` is
the lower id, and `normal` points from `a` to `b`. `point` is the deepest
contact point.
`relativeVelocity` (m/s) is `a`'s velocity relative to `b` at `point` as
that step began: each body's linear velocity plus its spin crossed with
the point's offset from its centre of mass. A static collider counts as
still, and a kinematic body moves at its velocity. Its part along
`normal` is the closing speed:
`Math.max(0, rv.x * n.x + rv.y * n.y + rv.z * n.z)`, which is positive
while they approach. `impulse` (N·s, at least 0) is the normal impulse the
solver gave the pair in that step, summed over substeps and contact points:
`b` took `impulse` along `normal` and `a` the same against it, so heavier
bodies hitting give more. Scale feedback on the closing speed (how fast)
or the impulse (how hard, mass included), as
`examples/rumpus-isle/src/critters.ts` does for its bumps.

```ts
const hit = og.physics.raycast(world, { origin: eye, direction: { x: 0, y: 0, z: -1 }, maxDistance: 30, filter: { mask: 0b10, exclude: player } });
if (hit.ok && hit.value) { /* hit.value.entity, distance, point, normal */ }
const under = og.physics.shapeCast(world, { shape: { shape: SHAPE_SPHERE, radius: 0.4 }, position: feet, direction: { x: 0, y: -1, z: 0 }, maxDistance: 2 });
const near = og.physics.overlap(world, { shape: { shape: SHAPE_BOX, size: { x: 4, y: 2, z: 4 } }, position: door });   // entities, by id
```

Push a dynamic body with `og.physics.applyForce(world, e, force, point?)`
(newtons over the next step: call it every tick for thrust, wind or a
car's downforce), `applyImpulse(world, e, impulse, point?)` (newton-seconds
at once: a kick, a hit, a jump; its velocity changes by impulse / mass),
`applyTorque` and `applyTorqueImpulse` (about each world axis; 2D uses z).
A point off the centre of mass spins it too. They apply at the og.physics
step after the systems, in the order given; a hit stop drops forces and
keeps impulses for after it; those not yet applied are in snapshots.
`og.RigidBody.centerOfMass` moves the centre of mass (a roly-poly toy, a
car that keeps its wheels down).

Inside a system the queries see the colliders as they were at the start of
the tick (`og.physics.queries` records them before the game's systems), so
the answer does not depend on system order; `QueryFilter.includeSensors`
adds sensors. The step itself runs on the lanes (§14) with the same result
as one thread. Beyond rigid bodies, all are components over the same simulation,
deterministic and part of snapshots (`reference/physics.md`):

- `og.Joint` — fixed, hinge (`JOINT_REVOLUTE`, limits, a velocity or
  position motor), slider, spring, rope, ball; to another body (`other`) or
  a world point (`otherAnchor`); `value` reads the angle or slide.
- `og.CharacterController` (3D, or top-down with `gravityScale` 0) — set
  `move` every tick and `jump` to jump; it climbs `maxSlope` and
  `stepHeight`, snaps to the ground, rides moving platforms, slides along
  walls, pushes dynamic bodies; read `grounded`, `velocity`, `ground`. Its
  og.Collider's `layers` and `mask` decide what blocks it, as for bodies
  (a ghost through a gate: leave the gate's layer out of its `mask`); the
  same for the PlatformerController.
- `og.PlatformerController` (2D) — `moveX`, `jump`, `down` in; jump height
  and time to apex, coyote time, jump buffer, wall jumps, one-way layers,
  corner correction as tunables (og-feel §12 has values by genre);
  `impulse` (m/s, taken once on the next step) knocks it back or launches
  it;
  `og.CharacterEvent` reports jumps and landings for dust and sounds.
  Both controllers push the dynamic bodies they move into (`pushBodies`,
  on by default) with at most `pushStrength` (1,000 N): the push is mass
  times the speed asked of the body, capped, so mass tells. Give a crate
  its `og.RigidBody.mass`: 2 kg is shoved at the run speed, 100 kg creeps,
  a tonne on the ground holds (friction beats the push). Lower
  `pushStrength` for a weak hero; `pushBodies: false` for a hero that
  never shoves (a crate then blocks like a wall).
  `templates/archetypes/runner`'s courier is one.
- `og.Vehicle` — a raycast car (3D, `throttle`, `steer`, `brake`; wheels
  from `og.physics.vehicleWheels`) or a car on wheel joints (2D). Its
  defaults drive a 1,000 kg road car: set the chassis's
  `og.RigidBody.mass` (og-feel §12 has the numbers for a racer, a truck
  and a kart). Its wheel rays meet only what its chassis's og.Collider
  `layers` and `mask` meet, so a ghost car on its own layer drives through
  the player's. `rollInfluence` 0 (the default) never tips it over in a
  bend; `og.RigidBody.centerOfMass` lowers its weight. It writes `speed`,
  `contacts` and `slip` (the most any tyre slides, 0 to 1); each wheel's
  `slip` and `sideSpeed` come with its pose, for smoke, skid marks and a
  squeal.
- `og.Rope` and `og.Cloth` — Verlet, drawn by the engine when `visible`.
  A cloth that folds or piles up (a dropped sheet, a cape on the ground)
  wants `thickness` about its `spacing` so it never passes through
  itself, `substeps` 2–4 and `friction` near 1 so it lies where it lands
  (self-collision costs about 1 ms a tick per 1,000 points).
- `og.Fluid2D` — a particle fluid drawn as metaballs on a sprite layer.
  Give two or more fluids the same `group` and they interact: by
  `density` the heavier sinks under the lighter (oil on water), by
  `mixing` 0 they stay in layers and by 1 they blend, colour and weight
  evening out where they touch; a group draws as one surface. Up to
  sixteen fluids draw a frame (four to a group).
- `og.Fluid3D` — a 3D particle fluid (3D profile) drawn as one
  screen-space surface that refracts and tints what lies behind it,
  reflects the sky and takes the scene's lights. Give it `bounds` (a
  container around the entity) or colliders and terrain to hold it; an
  emitter (`emitRate`, `emitOffset`, `emitVelocity`, up to `maxCount`) pours
  and a sink box (`sinkSize`, `sinkOffset`) drains. Groups work as in 2D:
  `density` 3 under 1 gives honey under water, `mixing` 1 blends their
  colours. Look: `color`, `opacity` per metre, `reflectivity`,
  `refraction`, `smoothing` (0 shows the beads); fluids drawn in one frame
  share one surface and the first fluid's look. Cost is the particles:
  about 0.9 ms of CPU a tick for 2,000 at one substep and 1.8 ms at the
  default two (native and wasm alike), 4 ms for 4,000 and 10 ms for 8,000
  at two, roughly double that when they pile deep, so budget a scene to a
  few thousand; the surface costs about 1.1 ms on the GPU whatever the count,
  growing with the pixels it covers. `og.physics.fluid3dParticles` reads
  the positions (the fx-showcase `fluids3d` station).
- `og.Ragdoll` — give the entity a rig
  (`og.physics.setRagdollRig(world, e, must(og.physics.ragdollRigFromSkeleton(world, anim.skeleton)))`),
  then set `weight` 1 to go limp. At weight 0 its bodies follow the
  animation's pose every tick, so the fall starts from the pose and its
  velocity; the model is drawn blended from the animation to the bones over
  `blendTime`. In a 2D world (`og.Physics.profile` 2) the same works for a
  side-on skinned model whose bones lie in the xy plane: the bones are
  rapier2d bodies on hinges about z (the fx-showcase `ragdoll2d` station).

Readouts (`ropePoints`, `clothPoints`, `fluidParticles`,
`fluid3dParticles`, `waterHeight`, `vehicleWheels`, `ragdollPose`) show
the last step. `templates/fx-showcase` has a station for each.

## 14. Performance

A frame is 16.7 ms at 60 Hz; keep step + render under half of it on a
desktop so phones have room. `og run --json` prints `advanceMs`, the
run's `frames` (step and render median, p95 and max) and its `systems`
(each system's mean milliseconds a tick, slowest first: the game's and the
engine's own, physics, the UI's end of frame, animation and the rest, which the
probe's `dump_world` also reports as `ms`), `og
polish` the p95 step and render time of every playtest (og-build §5), and
`og run --device` the first frame and frame times in a real browser or
phone (og-build §6). The usual costs:
draws (share geometry and one material per look), particle systems (cap
`maxParticles` in the emitter's .ogt, despawn emitters that are done),
per-tick TypeScript over many entities (iterate archetype columns, never
`world.get` per entity; spread expensive work over ticks, as og.ai's
`thinkEvery` does), and loads in the middle of play (load in `start`).
Point and spot lights are clustered (Forward+): a pixel shades only the
lights whose `distance` reaches it, so a few hundred lamps, torches or
neon signs cost little; keep each `distance` as short as the look allows.
Up to `MAX_LIGHTS` (256) in view are drawn, the brightest kept past that,
and up to `RenderCapabilities.maxShadowedLights` with `castShadow` (8 on
the full tier, 4 on WebGL2) cast shadows (shadows are the expensive part;
§16).
Engine work spreads over the cores on the lanes: physics from about 500
bodies, skinning, extraction and culling, with results identical to one
thread, so world hashes and replays do not change. Game systems stay on
the engine thread. On the web the lanes need cross-origin isolation
(`publish { isolation preferred }`, the default, in `project.ogt`; og-build
§7); without it the same game runs on one thread.
Simulated matter is paid for per point on the CPU every tick: a 3D fluid's
particles (about 1 ms per 1,000 at two substeps) and a self-colliding
cloth's points (about 1 ms per 1,000) are the ones to count (§13).

## 15. Looks: the frame's art direction as data

A **look** is an `.ogt` of `type look` under `assets/looks/` that sets the
whole treatment of the frame at once: the resources `RenderSettings`,
`PostEffects`, `Stylise`, `MaterialStyle`, `Fog`, `Sky` and `Environment`
(one block each, fields by name), the UI style (`ui`) and the `antialias`;
and, in the same file, the art contract the assets hold (below).
Choosing one for a premise is og-taste §1 and §12; this is the mechanism.

```text
type look
version 1
---
name "Harbour"
extends "anime-cel-3d"           # a built-in preset (templates/looks/<slug>.ogt) or another look's id
ui "styles/harbour/style.ogt"     # the game's own UI style, or a built-in by slug (ui "fantasy")
Stylise {
  ink true
  inkWidth 1.5
}
MaterialStyle {
  shadowTint "#3b3566"           # sRGB written, stored linear
  shadowTintAmount 0.6
}
```

```ts
export default og.game({ look: "looks/harbour.ogt", start, systems });   // GameDesc.look: applied after the first scene, before start

const flashback = must(og.load.look(world, "looks/flashback.ogt"));      // at any tick
must(og.render.applyLook(world, flashback));
og.render.look(world)?.name;                                              // the look last applied (LookInfo)
```

- Applying a look replaces each resource it names **whole** (the fields it
  sets, the rest their defaults; `PostEffects.flash` and `shakeTrauma`
  keep theirs) and leaves the others alone, then configures the UI with
  its style. Write per-game tweaks in `start` after it, or in the look. A
  system that applies one declares those resources among its writes.
- `extends` flattens at cook time, field by field (a field written wins);
  `assets/looks/<slug>.ogt` of the game's own replaces the preset of that
  slug for every look that extends it. The cook refuses an unknown field,
  a value out of range and an asset not in the bundle, naming the line.
- `og.load.lookInfo` (and `og.render.look`) answer `LookInfo`: `name`,
  `extends`, `resources`, `style`, `antialias`. A look's `antialias` (tier,
  off, msaa4 or taa) is the renderer's when `GameDesc.render` names none.
- **`og.Stylise`** moves the frame away from the photographic, every
  primitive off at zero: the pixel grid (`pixelHeight`, `pixelUpscale`,
  `pixelSnap`), fewer tones or a fixed palette through a dither
  (`posterize`, `palette` from `og.load.palette` of a `.hex` or `.gpl`,
  `dither`, `ditherStrength`), `halftone`, `hatching`, painterly
  smoothing (`paintRadius`, `paintEdgeDarkening`, `paintBleed`,
  `paintCanvas`), `ink` lines (with `inkDepth` for silhouettes and
  creases), `depthBands`, and early-3D `vertexSnap`, `affineTexturing`
  and `depthBits`. The UI and screen sprites stay unstylised.
- **`og.MaterialStyle`** sets every lit material that leaves the field at
  its default: `toon` (with a `gradientMap`), `shadowTint` and
  `shadowTintAmount`, `shadowDarkness` (1: every shadow a solid black
  mass, no sun and no ambient light there), `rimIntensity`, `outlineWidth` (with
  `outlineDistance` to thin lines with depth) and `flatShading`. Unlit and
  shader materials, sprites and the UI are untouched, and so is a material
  made with `styled: false` (exactly as declared under any look). The
  style's `toon` passes by a material with a physical lobe (`clearcoat`,
  `sheen`, `transmission`, `iridescence`, `anisotropy`, an `ior` other
  than 1.5), which keeps its physical shading and takes the rest; give it
  `toon: true` itself to make it toon anyway. An outline is a hull
  outline: each object's silhouette and its creases against other objects
  (the ground under it), never the edges between its own faces, so `.vox`
  models and flat-shaded meshes keep clean faces; for lines on every
  crease, `Stylise.inkDepth`.
- Built-in assets a look may name without owning a copy:
  `"@canvas/paper"`, `"@canvas/linen"`, `"@canvas/watercolor"` for
  `Stylise.paintCanvas`, and `"@palettes/lcd2"`, `"@palettes/handheld4"`,
  `"@palettes/dusk16"`, `"@palettes/bright32"`, `"@palettes/1bit"` for
  `Stylise.palette`.
- `RenderCapabilities.stylise` (`og.render.capabilities(world)`) names the
  primitive groups this device draws; a look using another still applies,
  without that primitive. A look in a streaming group applies once the
  group is resident (§6).

### The art contract: what the assets hold

The render resources treat the frame; seven more resources of the `art`
module, set in the same look, say what the game's **assets** must be
(docs/art.md in the engine repository): `og.ArtPalette` (the palette the
texels sit on: `colors` a palette asset or `"@palettes/<name>"`, `strict`
indexed colour, `maxColors` per asset, `rampSteps` per hue, `accents`
kept for signals by palette index, `dither`), `og.ArtPixels` (2D: `unit`
logical pixels per metre, `logicalHeight`, `spriteSizes` the allowed cell
sides, `rotation` free, nearest, cleanEdge or prerotated with
`rotationSteps`, `snap`), `og.ArtTexels` (3D: `density` texels per metre,
`maxSize`, `filter`, `mips`), `og.ArtShape` (`propFaces` and
`characterFaces` triangle budgets, `normals` smooth, flat or angle past
`normalAngle`, `bevel`, `voxel` edge, `silhouette` thinnest feature),
`og.ArtLine` (`width`, `color`, `place` outside, inside or both,
`distanceFalloff`), `og.ArtMotion` (`stepFps` for animation on twos,
`maxFrames` per clip, `squash`) and `og.ArtCamera` (`projection`,
`fixedAngle` with `pitch` and `yaw`, `fovMin`/`fovMax`, `pixelPerfect`).
Every field, with its range and unit, is in `reference/art.md`.

```text
extends "16bit-pixel"            # the preset carries the style's contract
ArtPalette {
  colors "palettes/harbour.hex"   # the game's own palette, under assets/
  maxColors 32
  accents 3 7                     # palette indices kept for the player and danger
}
ArtPixels {
  unit 16                         # 16-pixel tiles one metre wide
  spriteSizes 16 32
}
ArtMotion {
  stepFps 12                      # on twos
}
```

- Every field is **no constraint at zero** (0, false, the first variant,
  an empty list): a look without these blocks constrains nothing, and a
  game made before them cooks and draws as it did. The presets carry each
  catalogue style's values (og-taste §12); the game's look overrides them
  field by field, as the frame's. Lists take one line (`spriteSizes 16
  32`), and the cook checks every field as it checks the frame's.
- `project.ogt`'s `look "looks/<id>.ogt"` names the look whose contract
  the cook reads (`create` writes it beside `GameDesc.look`); the cook's
  report on that look lists the art resources it sets. Without the line
  the cook reads no contract.
- They are resources, so game code reads the contract too (a system lists
  them among its `reads`): `must(world.resource(og.ArtPixels)).unit` to
  snap a sprite, `og.ArtPalette`'s `accents` to colour the player. Setting them changes neither
  the frame nor the simulation; `og.load.lookInfo(...).resources` names
  them among the look's blocks.
- A system that applies a look writes only the art resources it declares
  among its writes; the others keep their values, so a system declaring
  only the frame's resources applies a preset with a contract unchanged.

## 16. Light, exposure, shadows and antialiasing

- **Tone mapping and exposure** (`og.RenderSettings`): `toneMapping` is
  `aces` (the default), `agx` (bright saturated lights go to white instead
  of shifting hue: neon, fire, magic), `neutral` (Khronos PBR Neutral:
  colours up to about 0.8 come out as picked and only highlights roll
  off; product shots, toy and cartoon colour that ACES darkens) or
  `none`; `toneMapped false` clamps
  so 2D colours come out as authored. `exposure` multiplies every camera's
  `Camera.exposure`. `autoExposure` meters each frame to middle grey;
  `adaptSpeedUp` and `adaptSpeedDown` (EV per second of game time, so a
  replay adapts the same) make it adapt like an eye: 1 up and 3 down,
  walking out of a cave into sun.
- **Environment maps** light PBR materials from a photographed sky: an
  `.ogt` of `type environment` beside an equirectangular `.hdr` or `.exr`
  (`source "studio.hdr"`, `size 128`, `intensity`, `rotation` in degrees,
  `samples`), prefiltered at cook time. Set it on `og.Environment`:

  ```ts
  const env = must(world.resource(og.Environment));
  env.map = must(og.load.environment(world, "env/studio.ogt"));
  env.rotation = 40;          // degrees about +Y; light and background turn together
  env.background = true;      // draw it behind the scene (Camera.background and og.Sky give way)
  ```

  `intensity` replaces `RenderSettings.environmentIntensity` while a map is
  set; `backgroundIntensity` dims the background alone;
  `StandardMaterialDesc.envMapIntensity` scales it per material. A look's
  `Environment` block sets it too.
- **Shadows**: the sun (`og.DirectionalLight`) casts three cascades;
  `shadowSoftness` (0.01–0.05) widens the penumbra with the distance from
  caster to receiver, as a real sun's does, and `cascadeBlend` (0.1–0.2)
  hides the seams between cascades. Point and spot lights with
  `castShadow` cast cube shadows on the full tier and paraboloid ones on
  WebGL2.
- **Antialiasing and renderer options** are `GameDesc.render`, read once
  when the renderer attaches:

  ```ts
  og.game({ render: { antialias: "taa", taaHistory: "catmullRom", taaSharpen: 0.3, pointShadows: "auto", shadowMapSize: 2048 }, ... });
  ```

  `antialias`: `tier` (TAA on WebGPU, Metal and Vulkan, 4x MSAA on WebGL2),
  `taa` (any tier, converges over about 8 frames: fast motion, thin
  geometry), `msaa4`, `fxaa` (one cheap pass after tone mapping; pixel and
  low-resolution looks), `off` (pixel art). `taaHistory: "catmullRom"`
  stays sharper under camera motion and `taaSharpen` (0–1) restores
  detail TAA softens. `RenderCapabilities.antialias` says what the device
  resolved it to.
- **Graphics quality** is RenderSettings' quality fields, read every
  frame: `rectShadows`, `shadowedLights`, `shadowMapScale`,
  `volumetricFog`, `softParticles`, `screenSpaceReflections`,
  `ambientOcclusion`, `aoResolution` (`postEffects`, or `full`/`half`
  in place of `PostEffects.aoResolution`), `ssrStepScale` and
  `volumetricStepScale` (multiples of the SSR steps and the volumetric
  fog's slices), `msaaSamples` (4, or 1 for FXAA where the
  antialiasing multisamples). `og.render.applyQuality(world, preset)`
  writes a preset's row of them and records it in
  `RenderSettings.quality` (`none` before any):

  | Field | low | medium | high |
  |---|---|---|---|
  | rectShadows | false | true | true |
  | shadowedLights | 2 | 8 | 8 |
  | shadowMapScale | 0.5 | 1 | 2 |
  | volumetricFog, softParticles, screenSpaceReflections, ambientOcclusion | false | true | true |
  | aoResolution | half | postEffects | full |
  | ssrStepScale, volumetricStepScale | 0.5 | 1 | 2 |
  | msaaSamples | 1 | 4 | 4 |

  Medium is the defaults, the frame drawn before any preset. The renderer
  reads the fields and nothing else, so `og run --commands
  "0.5:get:og.RenderSettings"` shows what is drawn, and a field set after
  the preset wins. The switches only take away and the scales only
  scale: the scene's PostEffects and lights keep their settings. High
  costs GPU time where the scene draws AO, SSR or volumetric fog (about
  twice each). The device caps two fields, which keep the row's value:
  `RenderCapabilities.maxShadowedLights` and `maxMsaaSamples` (1 where
  the antialiasing resolved to TAA, FXAA or off, so 4 draws one sample). The settings screen applies the
  player's pick when given the world (`og.ui.settingsScreen(id, { world,
  settings, ... })`); apply a saved preset yourself at start:

  ```ts
  start(world) { og.render.applyQuality(world, save.settings.quality ?? "medium"); }
  ```

  Inside a system, declare `og.RenderSettings` among its writes.

## 17. Saving progress

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

## 18. Geometry made at run time, lines, points and debug draw

Meshes need not come from a file. `og.render.bufferGeometry(world, desc)`
takes three.js BufferGeometry's attributes: `positions` (a Float32Array of
x y z), and optionally `normals`, `uvs` (v down the image), `colors`
(r g b or r g b a, multiplied into the material), `tangents` and `indices`
(a Uint32Array, counter-clockwise from the front). Missing normals are
computed as `computeVertexNormals` does. The geometry draws as any other
through `og.Mesh`, and it is a `Collider.mesh` at once (`physics` is true by
default), so a procedural level collides with what is drawn:

```ts
const terrain = must(og.render.bufferGeometry(world, { positions, colors, indices, dynamic: true }));
world.insert(e, og.Mesh, { geometry: terrain, material });
world.insert(e, og.RigidBody, { kind: BODY_STATIC });
world.insert(e, og.Collider, { shape: 5 /* SHAPE_MESH */, mesh: terrain });
// Each tick (water, a deforming surface): every attribute again.
must(og.render.updateGeometry(world, terrain, { positions, colors, indices }));
```

Only a geometry made with `dynamic: true` can be updated. The same vertex
and index counts rewrite it in place, and other counts reallocate it. Its
physics mesh is re-registered, and its colliders are rebuilt at the next
step. `og.render.geometryInfo` answers the counts and bounds of any
geometry.

Beside `boxGeometry`, `planeGeometry` and `sphereGeometry`, three.js's
primitives keep their parameters and defaults: `cylinderGeometry`,
`coneGeometry`, `capsuleGeometry` (an og.Collider capsule of the same
radius and `height` matches it), `torusGeometry`, `ringGeometry`,
`circleGeometry` and `tubeGeometry`. A tube follows a centripetal
Catmull-Rom curve through `points`, for pipes, rails and cables; give it
`path` instead (an og.path from `og.path.create`, or a scene's `path`
block found by `og.path.find`) and it follows that path's own curve,
Bezier or linear included, closed when the path is. Its rings are evenly
spaced along the length, as three.js spaces them. Rings and circles are one-sided, so use
`side: "double"` to see them from behind. `wireframe: true` on a
standard material draws the triangle edges instead of the faces, one pixel
wide on every tier.

**Shapes, extrusions, lathes and solids.** Three.js's modelling classes
keep their names, parameters and defaults, so a three.js extrusion is a
paste. `og.Shape` and `og.Path2` are three.js's Shape and Path:
`moveTo`, `lineTo`, `quadraticCurveTo`, `bezierCurveTo`, `splineThru`,
`arc`, `absarc`, `ellipse`, `absellipse`, `closePath`, `holes` and
`getPoints(divisions)`. They flatten to points before they cross to the
engine, so a points array works wherever a shape does
(`reference/shapes.md`).
- `shapeGeometry` is a flat shape with holes, one-sided like a circle.
- `extrudeGeometry` pushes shapes along +Z (`depth`, `steps`) or along an
  og.path (`extrudePath`). `bevelEnabled`, `bevelThickness`, `bevelSize`,
  `bevelOffset` and `bevelSegments` round its edges, as in three.js.
- `latheGeometry` turns a profile of `points` round +Y (`segments`,
  `phiStart`, `phiLength`), for vases, bottles and columns.
- `polyhedronGeometry` takes `vertices`, `indices`, `radius` and `detail`.
  `icosahedronGeometry`, `dodecahedronGeometry`, `octahedronGeometry` and
  `tetrahedronGeometry` take `radius` and `detail`.
- `sweepGeometry` sweeps a 2D `section` along an og.path. Its `scale` and
  `twist` are keyed over the path's length from 0 to 1, for horns, tusks,
  handles and twisted columns.
- `mergeGeometries` places `parts` (each a geometry with `position`,
  `quaternion`, `scale` and a `mirror` axis) into one draw. Write an
  array as the same part at several placements.
- `csgGeometry` is `union`, `subtract` or `intersect` of two placed
  closed geometries. Chain calls for more operands.

Every result is a geometry like `bufferGeometry`'s, with a BVH, `physics`
and `dynamic`, and answered by `geometryInfo`:

```ts
const gear = new og.Shape().absarc(0, 0, 1, 0, Math.PI * 2, false);
gear.holes.push(new og.Path2().absarc(0, 0, 0.3, 0, Math.PI * 2, true));
const disc = must(og.render.extrudeGeometry(world, {
  shapes: gear, depth: 0.2, bevelEnabled: true, bevelThickness: 0.05, bevelSize: 0.05,
}));
const vase = must(og.render.latheGeometry(world, {
  points: [{ x: 0, y: 0 }, { x: 0.5, y: 0.1 }, { x: 0.3, y: 1 }, { x: 0.4, y: 1.4 }],
  segments: 32,
}));
const cube = must(og.render.boxGeometry(world, { width: 1, height: 1, depth: 1 }));
const ball = must(og.render.sphereGeometry(world, { radius: 0.15 }));
const dice = must(og.render.csgGeometry(world, {
  op: "subtract", a: { geometry: cube }, b: { geometry: ball, position: { x: 0, y: 0, z: 0.5 } },
}));
```

The generators and the CSG decide every cut with exact predicates and use
no platform trigonometry, so every host makes the same triangles. A CSG
operand must be closed (every edge shared by two triangles). Its cost
grows with the product of the two triangle counts where their boxes
overlap, so cut low-poly operands at load, not every tick.

Every generator above takes `uvRepeat` as `boxGeometry` does (its uvs
scaled, so a texture tiles). `bevelSize` left out (or null) is
`bevelThickness - 0.1`, as in three.js: 0.1 at the default thickness.

**Models written as text: `type model`.** A prop made of these parts
belongs in an asset, not in `start`: an `.ogt` of `type model` under
`assets/` names each part's geometry by its `og.render` call, with the
desc's field names and defaults, places it as `mergeGeometries` places a
part, gives it a material and hangs it from a named pivot. `og cook`
builds it into the same model blob a glTF cooks to: meshopt, levels of
detail with their measured errors, the raycast BVH, the nodes. Load it
with `og.load.gltf(world, "models/clockwork.ogt")` and `world.instantiate`,
or name it in a scene (`model "models/clockwork.ogt"`), as any model.

```
type model
version 1
---
material "brass" {
  color 0.78 0.52 0.16       # linear, as glTF's baseColorFactor
  metalness 1
  roughness 0.32
}
material "glaze" {
  color 0.12 0.34 0.72
  roughness 0.3
}
pivot "axle" {               # a node: code and animation find it by name
  position 0 1.25 0
}
part "gear" {
  pivot "axle"               # hangs from the pivot (default: the root)
  material "brass"
  position 0 0 -0.1
  extrudeGeometry {
    shape {                  # og.Shape's calls, one a line
      moveTo 1 0
      absarc 0 0 1 0 6.283185307179586 false
      hole {
        absarc 0 0 0.28 0 6.283185307179586 true
      }
    }
    depth 0.16
    bevelThickness 0.04
    bevelSize 0.03
  }
}
part "bolt" {                # one mesh under three nodes: drawn instanced
  pivot "axle"
  material "brass"
  cylinderGeometry {
    radiusTop 0.06
    radiusBottom 0.06
    height 0.36
  }
  at {
    position 0.55 0 0
    quaternion 0.7071068 0 0 0.7071068
  }
  at {
    position -0.275 0.476 0
    quaternion 0.7071068 0 0 0.7071068
  }
  at {
    position -0.275 -0.476 0
    quaternion 0.7071068 0 0 0.7071068
  }
}
part "vase" {
  material "glaze"
  position -2.2 0 0
  latheGeometry {
    points 0 0 0.32 0.04 0.42 0.4 0.24 0.95 0.3 1.25
    segments 40
  }
}
part "horn" {
  position 2.2 0 0
  sweepGeometry {
    section {
      absarc 0 0 0.22 0 6.283185307179586 false
    }
    path {                   # og.path's lines: kind, closed, tension, point
      kind catmullRom
      point 0 0 0
      point 0.15 0.6 0
      point -0.45 1.35 0.15
    }
    scale 0 1                # t value, repeatable (twist the same)
    scale 1 0.05
  }
}
part "dice" {
  position 1.1 0.3 0.55
  csgGeometry {
    op subtract
    a {                      # an operand: a geometry block, placed
      boxGeometry {
        width 0.6
        height 0.6
        depth 0.6
      }
    }
    b {
      position 0 0 0.3
      icosahedronGeometry {
        radius 0.08
        detail 1
      }
    }
  }
}
```

- Geometries: `boxGeometry`, `planeGeometry`, `sphereGeometry`,
  `cylinderGeometry`, `coneGeometry`, `capsuleGeometry`, `torusGeometry`,
  `shapeGeometry`, `extrudeGeometry` (`extrudePath { }` a path block),
  `latheGeometry` (`points x y x y ...`), `polyhedronGeometry`, the four
  solids, `sweepGeometry`, `csgGeometry` (`a { }`, `b { }`) and
  `mergeGeometries` (`part { }` blocks), each with `uvRepeat u v`. A
  part holds exactly one; combine more with `csgGeometry` or
  `mergeGeometries`.
- Shapes: `moveTo`, `lineTo`, `quadraticCurveTo`, `bezierCurveTo`,
  `splineThru x y ...`, `arc`, `absarc`, `ellipse`, `absellipse` (angles in
  radians, then `true` for clockwise), `closePath`, `points x y ...`
  (setFromPoints) and `hole { }`. They flatten to the points og.Shape
  gives in TypeScript, bit for bit.
- Placement: `position`, `quaternion x y z w`, `scale`, `mirror x|y|z`,
  on the part or in `at { }` blocks, one node each. The same part at
  several placements is one mesh drawn instanced; a mirrored placement is
  baked into a mesh of its own.
- Materials: `color`, `opacity`, `metalness` (default 0), `roughness`
  (default 1), `emissive`, `emissiveIntensity`, `doubleSided`,
  `alphaTest`. A part without one draws glTF's grey default.
- `og cook --report` lists the model's `triangles`, `parts`, `draws` (the
  distinct meshes; instanced placements are one) and `bvhBytes`/`bvhShare`.
  Past `budgets.triangles` or `budgets.vertices` in project.ogt the cook
  refuses it, so lower `segments`, `curveSegments`, `steps` or `detail`.
  Import settings (`models/clockwork.ogt.ogt`, `model { lods ... raycast
  false }`) apply as to a glTF.
- `og dev` re-cooks it on an edit and the running game draws the edit
  under the same asset handle.

**Lines and points.** `og.render.lines(world, desc)` and
`og.render.points(world, desc)` are immediate mode, like
`og.render.sprite`: submit them every tick they should show.
- Lines: `mode` is `strip`, `segments` or `loop`. `width` is in CSS
  pixels, or in world units with `worldUnits`. They also take `colors` per
  point, `depthTest` (false draws over the scene), `dashed` with `dashSize`
  and `gapSize`, and `screen: true`. Screen lines use CSS pixels from the
  top left of the play surface and draw over the finished frame, under
  the UI.
- Points: `sizeAttenuation` makes `size` a size in world units; without it
  `size` is a fixed size in CSS pixels.
- A static set is data: the `og.Lines` and `og.Points` components draw a
  geometry's vertices in order at the entity's transform. Make the
  geometry with `bufferGeometry({ positions, colors, physics: false })`,
  unindexed, any count.

**What is under the cursor.** `og.render.raycast(world, ray, desc)` tests
what is drawn (every og.Mesh and og.Model at its world transform), not the
colliders. It goes through a BVH per geometry and answers every hit,
nearest first; `first: true` answers only the nearest. Each hit has
`entity`, `geometry`, `distance`, `point`, `normal` (the face's, in world
space), `uv` and `faceIndex`. Inside a system it reads the meshes as the
tick started, as `og.physics.raycast` does. Use it for picking, decals on
detailed meshes, and line of sight against geometry without colliders.

A model's BVH is cooked: `og cook` builds each primitive's into the
model's `.ogb` (about 6 to 9 bytes per triangle compressed, listed as
`bvhBytes` in `og cook --report`), and it loads with the mesh, so the
first raycast never stalls. A model no ray is cast at can leave it out
with import settings; its first raycast then builds it:

```
type import
version 1
---
model {
  raycast false
}
```

Leave it on for anything a player can click, aim at or stand a decal on.
Turn it off for a large model that is only scenery (templates/game's
16,512-triangle dropship gains 108 KB, 64 % of its blob). Geometries made
at run time (`bufferGeometry`, the primitives) build theirs when made or
updated.

**Debug draw.** `og.debug.line`, `box`, `sphere`, `arrow`, `axes` and
`text` (a label above a world point, in the UI's body font; every field is
in `reference/debug.md`) each take
`{ color, width, duration, rotation }`. `duration` is in ticks; 0 shows the
shape this frame only, so draw it every tick. Debug draws show over the
finished frame and never hide behind geometry, so they read in every
capture. The command channel switches the views:
- `debug:physics` shows the colliders, joints and contacts.
- `debug:nav` shows the grids' blocked cells, the navmesh polygons and
  every agent's path.
- `debug:bounds` shows every drawn mesh's box.
- `debug:none` shows only the draws; `debug:off` and `debug:on` hide and
  show everything.

`og.debug.setView` does the same from code, and `RenderOptions.debugView`
picks a view from the first frame. A publish build drops all of it, so
debug draws may stay in the code. Debug draws are not world state:
snapshots and replays do not carry them.

## 19. Video: footage in, videos out

**Footage in.** An `.ogt` of `type video` names an `.mp4`, `.webm` or
`.mov`; `og cook` turns it with ffmpeg (`og doctor` says whether ffmpeg
and ffprobe are there) into a texture array of its frames and its sound:

```text
type video
version 1
---
source "clips/intro.mp4"   # beside this file, or under assets/
fps 30                     # frames kept per second (default 30)
maxSeconds 8               # what is kept from the start (default 8)
size 320 180               # each frame's size (default the source's, at most 1024 wide)
kind color                 # or pixel: nearest, lossless
audio true                 # its sound track (default true)
```

At most 256 frames (fps × seconds): the cook refuses more, naming the
`maxSeconds` that fits, and warns past 64 MiB of cooked frames. Keep
footage small: a 320×180 clip of 8 s at 30 fps is about 2 MiB cooked.
Load it in `start`, then play it with `og.VideoPlayer`; its frame is a
plain `Texture`, so a sprite, a material map or a UI image shows it:

```ts
const info = must(og.load.video(world, "clips/intro.ogt"));
const screen = must(world.spawn({ name: "screen", position: { x: 0, y: 0, z: 0 } }));
must(world.insert(screen, og.Sprite, { texture: info.texture, width: 16, height: 9 }));
must(world.insert(screen, og.VideoPlayer, { video: info.video, loop: true }));
must(world.insert(screen, og.AudioSource, {})); // its sound, kept in step
```

The `og.video` system plays it by the tick (not the time scale, so the
picture stays with its sound): with a 30 Hz tick, tick 90 shows the
frame 3 s in, the same in every run and after a restore. Write
`VideoPlayer.time` to seek (the sound starts again there on the next
tick), `playing` to pause, `speed` for slow motion (the pitch follows).
`frame` and `finished` are written for you, and so is `texture`: each
player of a video has a texture of its own (the first takes
`info.texture`, each further one a texture made for it over the same
frames), so two players at different times show different frames. With
more than one player, draw each player's `VideoPlayer.texture`. Every field and the loaders are in `reference/video.md`.

**Videos out.** `og render` plays a command script headless and makes a
video of it: a frame every `tick / fps` ticks (the fps divides the tick
rate; the error names the nearest that do), read back without the
probe's round trip.

```sh
og render . --commands "0.5:type:s;2:key_down:Space" --fps 30 --to 8 \
  --size 1280x720 --out og-out/intro.mp4 --audio --contact-sheet 6 --jobs 4
```

It writes `intro.mp4` (H.264; `.webm` is VP9) with the run's audio,
`intro.frames.json` (each frame's tick, time and SHA-256, and the
audio's), and `intro.contact-sheet.png`: a grid of frames over the range,
each labelled with its time and tick. **Review the contact sheet first**:
one image shows whether the scene moves as the script meant, without
playing anything. `--out <dir>` (or `--frames png|jpg`) writes the
frames too; without ffmpeg the frames are what you get. Two runs of one
script give the same SHA-256s, so a frame that changed is a change you
made. `--jobs N` splits the range over processes (each starts from a
snapshot of the world) and gives the same frames, faster.
`--timeline <name>` renders a timeline (`type timeline`) over its
duration once timelines exist (stream S2 of docs/depth.md); until then
use `--commands`. On a CPU-only machine (lavapipe) a 1280×720 frame of the
3D template takes about 0.4 s in one job, 2.6× less with `--jobs 4`:
render at 640×360 (about 0.1 s) to iterate, full size once.

## 20. Two players over a room: lockstep

To play together, every peer runs the whole world and only inputs cross
the room (`reference/net.md`; the rooms themselves are og.platform's, taught by
the og-platform-sdk skill). Follow three rules:

1. In `start`, set `og.Lockstep`: `enabled = true`, `players`,
   `inputDelay` (2 on a LAN, 4 to 6 over the internet) and `hashEvery`
   (10). Spawn one entity per player with `og.PlayerInput { player: seat }`.
2. Read `og.Input`, `og.Gamepad` and the UI's touch controls in **one**
   system, and hand them over with
   `og.net.submit(world, { x, y, buttons, aimX, aimY })` every tick.
3. Every system that changes the world reads `og.PlayerInput` (its `x`,
   `y` and `buttons` bits), the world and `og.Rng`, and nothing else: no
   `Math.random()`, no `Date.now()`, no module variables, no
   `og.RoomMessage`. Draw per-peer things (whose turn it is, "waiting")
   with og.ui only, and keep them out of the world.

```ts
const Submit = defineSystem("Submit", { reads: [og.Input] }, (_q, _t, world) => {
  const input = must(world.resource(og.Input));
  must(og.net.submit(world, { x: input.axis("KeyA", "KeyD"), y: input.axis("KeyS", "KeyW"), buttons: input.down("Space") ? 1 : 0 }));
});
const Move = defineSystem("Move", { reads: [og.PlayerInput], writes: [og.Transform] }, (q, time) => {
  for (const a of q.archetypes()) {
    const pin = a.column(og.PlayerInput);
    const p = a.column(og.Transform).position;
    for (let i = 0; i < a.count; i++) p[i * 3] += pin.x[i]! * 6 * time.dt;
  }
});
```

Alone, or with lockstep off, `submit` drives the local player on the next
tick, so the same code plays single-player. Natively, run `og relay` and
then `og run . --room ws://127.0.0.1:47900/<room>` in two terminals; on
the web the shell carries the platform's rooms, and a Play link with
`?room=<id>` puts the page in that room at load (the game hears
og.RoomJoined as for its own roomJoin; `og.platform.roomInfo(world).link`
names the link's room), so a game that plays over a room need not ask for
one when the link named it. A playtest plays itself in
a room with `room: { players: 2, commands: [...] }`: its runs are `run`
and `run.peers`, and each has `netHash` and `lockstep`. Equal `netHash`
values at the same tick mean the peers stayed in step. Once lockstep runs
a script's steps are on the room's shared ticks (a peer that took the
authority's snapshot keeps its steps on theirs), so end each script with
`quit` (or give `room.ticks`) and every place stops on one tick; a `then`
step relaunches every place at once into a room of its own, its runs in
`run.then[n]` and `run.then[n].peers`.

When the worlds part, `og.Desync { tick, peer }` fires once and
`og.net.status(world).desync` holds the tick. Then
`--commands "...:dump_desync:d.json"` names the first divergent tick and
the entity, component and field that differ, with both values: fix the
system that writes that field. The archetype `templates/archetypes/arena`
(Coin Duel) is the worked example.

## 21. Checking a run: describe, expect, trace, write-back, budgets

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

## 22. Levels of detail, reflection probes and screen-space reflections

**Levels of detail.** Every model the cook reads is simplified into
levels: by default `lods 1 0.5 0.25`, each level a fraction of the full
mesh's triangles. A glTF's own `MSFT_lod` levels are kept instead. Import
settings change them per model, and `lods 1` keeps only the full mesh:

```
type import
version 1
---
model {
  lods 1 0.4 0.15
  lodError 0.08
}
```

The cook measures each level's error, the largest distance between the
level and the full mesh in the model's units, and `og cook --report`
lists each model's triangles (`lods`) and error (`lodErrors`) per level.
The renderer draws each `og.Model` entity at the coarsest level whose
error covers less than a pixel at the entity's distance, with
hysteresis, so nothing flickers at a boundary: a small prop keeps its
full mesh until it is far away, a large tree drops levels sooner. A
level whose error is too large for its purpose (a hard-surface model
melting at a distance) is cooked tighter with `lodError`, or the model
keeps more detail with `lodBias`. `og.Model.lodBias` (per entity) and
`RenderSettings.lodBias` (the frame) scale that choice: 2 keeps the full
mesh twice as far, and 0 always draws it. Check the choice with
`debug:lod`, which tints models green, yellow, orange and red by level,
with `dump_entity:<name>` (`render.lod`), and with the triangles in
`ProbeTimings.triangles` or `dump_render`.

**Reflection probes.** Smooth surfaces reflect the environment map, which
is the sky. Indoors that is wrong: a chrome sphere in a room would show
the sky. Put an `og.ReflectionProbe` in the room, and lit surfaces inside
its box reflect a cube captured from its position instead, box-projected
so the walls reflect where they are:

```ts
const probe = must(world.spawn({ name: "hall-probe", position: { x: 0, y: 1.6, z: 0 } }));
world.insert(probe, og.ReflectionProbe, {
  size: { x: 10, y: 4.2, z: 10 }, // the room's extents, centred on the entity
  resolution: 128,                // texels per cube face
  mode: "baked",                  // or "realtime", every `interval` ticks
  blendDistance: 0.5,             // fade into the environment at the box's faces
});
```

- Choose the probe's mode by what moves in the room:
  - A `baked` probe is captured once, on its first frame. For a static
    room, run `og bake probes [project] --commands "0:station:hall"` (the
    commands reach the scene). It writes the captures into
    `cooked/probes/`, and runs then load them instead of drawing them.
    A bake is keyed by the probe's place and size and by its room's
    content: moving the probe or changing what is in its box (a mesh,
    a material, a texture, a light) needs a new bake, and until then it
    draws its own capture. `og doctor --project` lists stale bakes
    (`stale-probe`).
  - A `realtime` probe redraws six faces every `interval` ticks. Keep
    the interval high.
- Size the box to the room. Each object takes the smallest box holding
  its centre, so two rooms that share a model each reflect their own.
  The capture shows everything in the room: characters in their pose
  (and each one takes the probe of the room its pose stands in), terrain,
  grass, water (refracting what lies below it and, on the full tier,
  reflecting the capture in screen space), 3D fluids, glass, particles
  (soft ones fading as in the frame), lines and world sprites and text,
  lit by the sun and every local light the camera's frame has (a light
  whose range the camera does not see lights nothing in the capture
  either). As in the frame, water and fluids show the opaque scene
  through them, not glass behind them. Terrain, grass and water inside
  the box are lit by it too. At most 8 probes (`MAX_REFLECTION_PROBES`) are used,
  nearest the camera first.
- `dump_render` lists the frame's probes and whether each was loaded from
  the cache. In a playtest, assert what the sphere shows with
  `expect:region:x,y,w,h:color:#rrggbb:tol`, and a model's level of
  detail and the frame's triangles with `expect:entity:<name>:render.lod==1`
  and `expect:timings:triangles<=1500000`.

**Screen-space reflections.** On the full tier, `PostEffects.ssr` adds the
frame's own reflections to smooth surfaces: a wet street, a polished
floor. They show what is on screen, including moving things:

```ts
const post = must(world.resource(og.PostEffects));
post.ssr = true;
post.ssrMaxDistance = 12; // metres a ray travels
post.ssrSteps = 48;       // more for long, thin reflections
post.ssrThickness = 0.3;  // how far behind a surface a ray still hits it
post.ssrFade = 0.15;      // fade at the screen's edges
```

SSR is weighted by roughness as the surface is shaded, its roughness map
and a terrain layer's roughness included (`roughness` 0.05 reflects
sharply; above 0.75 it reflects nothing), and by Fresnel. What it misses (off screen,
behind objects) falls back to the probes and the environment, so use SSR
together with a probe. WebGL2 draws no SSR: check
`og.render.capabilities(world).ssr`, and on that tier the probe is the
whole reflection.

## 23. Render targets, second cameras, layers, custom shaders and post passes

**Render to a texture.** `og.render.renderTarget(world, { width, height,
format, depth })` makes a target, as three.js `WebGLRenderTarget`. It
answers `{ target, texture, depthTexture, width, height }`. `texture` is
an ordinary Texture: a material's `map` or `emissiveMap`, an og.Sprite, an
`og.render.sprite` or an og.ui image shows it. `format` is `ldr` (the
default) or `hdr`:
- `ldr`: 8-bit and finished (exposed with the view's `exposure`, tone
  mapped). Use it for a monitor, a minimap or a portrait, shown through an
  unlit material's `map`.
- `hdr`: linear radiance, exposed once with the frame that shows it. Use
  it for a mirror or a portal, shown as `emissiveMap` on a material whose
  `color` is black.

`depth: true` also keeps the view's linear depth, from near (0) to far
(1), of its opaque meshes, on every tier, WebGL2 included. It costs a
second draw of the view's opaque meshes, so ask for it only when a
material reads it. (A scene with soft particles, `softDistance` above 0,
pays that draw in every view anyway: its particles fade against it.)
`RenderCapabilities.maxRenderTargetSize` is the largest side.

**A second camera.** `og.CameraView` on an entity is a camera with
og.Camera's fields (`fov`, `near`, `far`, `zoom`, `orthographic`,
`height`, `exposure`). It looks down the entity's local -Z:
```ts
const screen = must(og.render.renderTarget(world, { width: 480, height: 270 }));
world.insert(monitorMesh, og.Mesh, { geometry: plane, material: must(og.render.standardMaterial(world, { map: screen.texture, unlit: true })) });
const cam = must(world.spawn({ name: "cctv", position: eye, quaternion: og.transform.lookAt(eye, target) }));
world.insert(cam, og.CameraView, { target: screen.target, fov: 60, every: 2 });
// Split-screen: no target, a rect of the frame (fractions from the top left).
world.insert(p2, og.CameraView, { viewportX: 0.5, viewportWidth: 0.5 });
```
- `every: n` draws it on every n-th tick, and the target keeps its last
  image in between.
- `order` sorts the views. Every view draws before the frame, so a
  material shows this frame's image.
- `clear` with `clearColor` replaces the sky or background colour.
- `active: false` stops it.

A view draws what the main camera draws, from its own eye and on its own
`layers`: meshes and models (skinned meshes and custom shaders included),
opaque then transparent back to front, the terrain and the sky, particle
systems, trails, og.Lines and og.Points, and world sprites and text, lit and
shadowed by the frame's lights. Each view is a second draw of everything
its layers admit, so keep it cheap with `layers` and `every`.
- An `ldr` target and a viewport are exposed with the view's `exposure`
  and tone mapped with the frame's curve. An `hdr` target holds linear
  radiance and is exposed by the frame that shows it.
- Bloom, AO, SSR, depth of field, colour grading and custom post passes
  run on the main frame only (the 2D glow, `PostEffects.glow`, is the
  exception: a view adds its own).
- In a view, sprites are 2D-lit and glow and 2D fluids and soft
  particles draw as in the main frame. Screen sprites, the UI and debug
  draw stay on the main frame.

A viewport view draws over the finished image and under the UI.

**A mirror.** Put `og.Mirror` on the glass's entity. It reflects the main
camera in the entity's xy plane (its +Z faces the room) and fits the image
to a `width` × `height` rect:
```ts
const rt = must(og.render.renderTarget(world, { width: 512, height: 384, format: "hdr" }));
world.insert(glass, og.Mesh, { geometry: must(og.render.planeGeometry(world, { width: 4, height: 2.4 })),
  material: mat({ color: { r: 0, g: 0, b: 0 }, emissive: { r: 1, g: 1, b: 1 }, emissiveMap: rt.texture }) });
world.insert(glass, og.Mirror, { target: rt.target, width: 4, height: 2.4 });
```

**Layers.** `layers` on og.Mesh, og.Model, og.Sprite, og.ParticleSystem,
og.Lines, og.Points and og.Camera (and on og.CameraView and og.Mirror) is a
three.js bit mask:
bit n is layer n, and the default 1 is layer 0. A camera draws an object
when `object.layers & camera.layers` is not 0. `og.render.raycast`'s
`layers` uses the same test. Some uses:
- A first-person body on layer 1 (mask 2) is hidden from the player's
  camera (mask 1) and shown on a third-person monitor (mask 3).
- A minimap marker only the minimap camera draws.

Shadow maps take every caster whatever its layers, so a hidden object
still casts a shadow. Give it `castShadow: false` if it should not.

The probe's `describe` lists what the main camera shows, and each
og.CameraView and og.Mirror under `views` with what its own image shows,
on its layers. An entity on a layer only a view sees is found under that
view, not in `entities`. A main-camera entity names the views that also
show it in its `views`. `pick` and `expect:entity:...:within` read the
main camera.

**Full custom shaders.** `mode full lit` (or `mode full unlit`) in a `type
shader` asset replaces the lit shader's stages with your own entries:
```wgsl
fn vertex(v: VertexInput, model: mat4x4<f32>, p: Params) -> VertexOutput   // v: position normal uv color tangent (object space; skin and morphs applied)
fn fragment(f: FragmentInput, p: Params) -> SurfaceOutput                  // f: world local normal uv color data view pixel time
// VertexOutput: world (projected by the engine), normal, uv, color, data (free, interpolated)
// SurfaceOutput: color (albedo + alpha; unlit: the colour), normal, metallic, roughness, emissive
// camera(): view_proj, position, forward, time. tex0(uv)..tex7(uv): the material's map, metalnessMap,
// roughnessMap, normalMap, emissiveMap, aoMap, gradientMap, dissolveMap. Helpers: standard_vertex(v, model),
// standard_surface(f), noise3(p).
```
An entry you leave out is the standard one.
- `lit` surfaces are lit by the sun's cascades, the point, spot and rect
  lights with their shadows and cookies (a rect light as an area, as on a
  standard material), the environment and the fog. `unlit` surfaces draw
  as returned.
- The depth prepass, every shadow pass and every view run the same
  entries, and alpha under the material's `alphaTest` is cut in each of
  them. A dissolve's shadow therefore has its holes.
- Make the material with `og.render.shaderMaterial` and set parameters with
  `og.render.setShaderParam`, as for hooks (§12).
- `templates/fx-showcase/assets/shaders/dissolve-full.*` is the example.

**Custom post passes.** A `type post` asset declares
`fn post(f: PostInput, p: Params) -> vec4<f32>` in its `code` or `source`,
with up to eight `param` lines. The function reads:
- `previous(uv)`: the pass before it, linear HDR;
- `scene_color(uv)`: the lit image;
- `depth(uv)`: view depth in metres;
- `normal(uv)`: the world normal;
- `f.uv`, `f.pixel`, `f.size` and `f.time`.

It returns linear HDR. To use one:
1. Load it with `og.load.post(world, "post/crt.ogt")`.
2. Put the handles in the list `PostEffects.custom`, at most
   `RenderCapabilities.maxCustomPosts` (8) of them. They run in its order,
   each over the one before it, after depth of field and motion blur and
   before exposure, bloom and tone mapping. Write the list whole:
   `post.custom = [grade, crt]`, `post.custom = [...post.custom, h]`,
   `post.custom = []`. Reading it answers a frozen copy, so
   `post.custom.push(h)` throws instead of changing nothing; nine passes
   are refused, never cut to eight. A look gives it on one line:
   `custom "post/tint.ogt" "post/crt.ogt"`.
3. Set its parameters with `og.render.setPostParam(world, post, "curve",
   [0.1])`.

The cook compiles it for every back end. A pass that a device still fails
to compile is skipped and listed by `og.render.postErrors(world)`.
`templates/fx-showcase/assets/post/crt.*` is a CRT example.

## 24. Baked lighting, volumetric fog, area lights and cookies

**Baked lighting.** Use it for light that bounces around a static space,
such as a red wall tinting the floor beside it or a room lit through its
door. Mark the scene's unmoving models `static true` and put an
`og.LightVolume` box over the space where dynamic objects move:

```
entity "hall" {
  model "models/hall.glb"
  static true
  Transform {
    position 0 0 0
    quaternion 0 0 0 1
    scale 1 1 1
  }
}
entity "hall-probes" {
  Transform {
    position 0 2 0
    quaternion 0 0 0 1
    scale 1 1 1
  }
  LightVolume {
    size 12 4 12
    spacing 1
  }
}
```

Then run `og bake lightmaps` in the project. Its options are
`--samples 256`, `--bounces 3`, `--texels-per-metre 16`, `--max-size 1024`,
`--sky r,g,b`, `--seconds S` and `--scene <id>`; the fx-showcase Cornell
box takes 1.4 s at 512 samples. The bake is a CPU path tracer and needs
no GPU. It writes `<scene>.lightmap.ogt` and `.ktx2` beside the scene;
commit both.

A static entity needs a name. The bake holds the light that bounces, the
sky and emissive surfaces; lights still shine and cast shadows live on
top. Static models draw with their lightmap, and everything else inside a
LightVolume takes its ambient light from the probes. After you change the
scene's text or a static model's geometry, the bake is stale and the scene
renders unbaked until you bake again; a re-cook that keeps the geometry
(a cooker update, a texture or raycast option) keeps the bake. `og doctor
--project` warns `unbaked`, and
`og.render.bakedLighting(world)` lists each scene with `baked` and a
`reason`. `RenderSettings.bakedLighting = false` draws everything unbaked,
which is useful to compare. Every instantiated scene draws its own bake,
so baked scenes loaded together or streamed in side by side all keep
their light.

**Volumetric fog.** Set `Fog.volumetric = true` to light the fog as a
medium, which gives shafts through windows and the cones of spots. Tune it
with `density`, `heightDensity` and `falloff` (thicker low down),
`scattering` (how bright) and `anisotropy` (towards 1, bright looking into
the light), up to `volumetricDistance`. Fog of 0.03 to 0.1 per metre reads
well indoors. With `og.DayNight`, the shafts move with the sun.
`RenderCapabilities.volumetricFog` is `froxel` on the full tier or
`raymarch` on WebGL2. On both, the sun, spots, points and rect lights light
the fog, so a lantern glows in it on WebGL2 too.
`RenderCapabilities.volumetricLights` is how many local lights light one
sample (64 for the froxel grid, 8 for the raymarch, brightest first).

**Area lights.** `og.RectLight { width, height, color, intensity,
twoSided }` emits from a rectangle facing the entity's -Z. Aim it with
`og.transform.lookAt`, as you would a spot. A glossy floor shows the
rectangle's shape in its highlight. Pair one with an emissive slab for
the fixture. Set `castShadow` for a soft shadow: sharp where a caster
touches the floor, widening with the panel's size away from it. It takes
one shadow layer (two when `twoSided`) from the same budget as shadowed
spots and points, and is drawn from the panel's centre, so a caster right
against a large panel is shadowed as from its middle.

**Cookies.** `SpotLight.cookie` is a texture projected along the cone,
like a window's lattice or stained glass. `PointLight.cookie` is six
square faces stacked top to bottom (+X, -X, +Y, -Y, +Z, -Z), like a
pierced lantern. A cookie multiplies the light and its colour. A frame
draws at most `og.render.capabilities(world).maxLightCookies` (24) cookie
layers: a spot cookie takes one, a point cookie six, and a texture shared
by lights of one kind counts once. Past that, the later lights shine
without their cookies and the log warns, naming them; share cookie
textures between lanterns to stay under it.

## 25. Where to look things up

`og api find <words>` and `og api show <name>` (og-core §7) answer one API
question at a time from the wiki below.

`reference/index.md` (the generated wiki shipped with this skill) maps every
module (grouped by skill, as in §1), type and function to its page, with the IDL text, defaults, ranges
and units of every field.

`guides/three-to-og.md` (beside this skill) maps three.js's classes and
idioms to the engine's: the scene graph to entities and og.Parent,
Object3D to og.Transform, the geometry classes to §18's generators, the
materials to StandardMaterialDesc, the loaders to og.load, the render
loop to systems on the tick, Raycaster to `og.render.raycast`, and the
gaps by name. `og migrate three <file.ts>` rewrites a three.js file's
mechanical part with those mappings and marks every line it cannot,
citing the guide's section.
