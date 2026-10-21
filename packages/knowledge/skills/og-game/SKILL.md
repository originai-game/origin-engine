---
name: og-game
description: Building a game on Origin Engine — the project layout, components and systems on the ECS, the game entry, scenes as .ogt text, assets cooked from assets/ to cooked/ (glTF and MagicaVoxel models), the og.load loaders, streaming groups, sprites, texture arrays and drop shadows, skeletal and stepped animation, effects and physical materials, looks (the frame's art direction as data: og.Stylise, og.MaterialStyle, presets), tone mapping, exposure, environment maps, shadows and antialiasing, fog, sky, day and night, weather and water, terrain, material shaders in WGSL, physics beyond rigid bodies (joints, character and platformer controllers, vehicles, ropes, cloth, 2D and 3D fluids, ragdolls) and its queries, and the performance budget; the entry point, with the map of the other nine skills and the modules each teaches. Use first, and when creating, structuring or extending a TypeScript Origin Engine game (a project with a project.ogt; not a Godot project.godot).
---

An Origin Engine project is plain files: TypeScript in `src/`, source assets
in `assets/`, text scenes, and a manifest. `npm create @origingame/game`
produces it; everything after that is code and text, never an editor. This
skill is where to start: what a game is made of, and which skill to read
next for each part.


## What works

Two games were written from this package alone: `examples/lantern-vale`
(a 3D top-down adventure) and `examples/hollowbell` (a 2D platformer).
What they asked for the documented way worked the first time: a Tiled map
with a blob47 auto-tiled terrain, Tiled's parallax and objects with
properties (§8); flipbook frame events driving footsteps, strikes and a
bell; texture arrays as world sprites, screen sprites and emitter layers;
streaming groups of `.vox` models and textures loading as the camera
neared them (§6); a look applied before `start` and a second one at a tick
for the results (§15); `og.physics.overlap` on a layer and with sensors;
a character controller shoving a pile of bodies (§13); a skinned glTF on
twos (§9). `og cook` took their 47 and 69 assets cold in 8.2 s and 3.6 s,
under 100 ms after a change, without a warning, and its refusals named
the line and the fix.

## Writing a game from this package

The order both games found productive:

1. **`project.ogt`** (§2): name, id, the engine pin, the profile, tick and
   seed. The scaffold writes it; change it there and nowhere else.
2. **`GameDesc`** in `src/main.ts` (§3): the scene, the look, the systems,
   and from the start a command for each state a test must reach (`play`,
   `warp:<where>`) and facts for what it must check.
3. **Assets by script**: `tools/make-assets.*` writes every texture,
   tileset, map, flipbook, emitter, sound and look deterministically (both
   games and every archetype do), with import settings beside each source
   (§5, §8, §10). Rerun it after a change; never hand-edit its outputs.
4. **Cook** (`og cook`, og-build §1): read every refusal, it names the line
   and the fix.
5. **Run with commands** (og-build §1, §4): `og run . --headless
   --commands "...;screenshot:x.png;...;quit"`, then open each frame and
   say what it shows. Do this after every change that shows on screen.
6. **Playtests with reference frames** (og-build §3): `tests/*.og.ts`
   asserting on facts, `get:` answers and dumps, one per feature, and a
   frames playtest comparing screenshots with `references/<tier>/`.
7. **Polish** (og-build §5): `og polish` until its findings are gone; each
   names the section that fixes it (og-taste §10, og-feel §13).

Three mistakes both games made before reading the right page:

- **The resource handle.** A system reads or writes a resource only when
  it lists it among its `reads` or `writes` (og-core §4: undeclared, it
  throws naming the system and the record). Fetch `world.resource(R)` in
  the call that uses it and pass that handle to the helpers it calls,
  rather than fetching it again in each one.
- **A look replaces resources whole.** `GameDesc.look` and
  `og.render.applyLook` set every field of each resource the look names
  (the rest to their defaults), so a per-game value written before the
  look is lost: write it in the look, or in `start` after it (§15).
- **Where probe outputs go.** `og run` writes screenshots and dumps under
  `og-out/` (`--out` chooses), `og test` each playtest's under
  `og-out/test/<name>/`, and a playtest reads them back by the name its
  step gave (`run.json("shop.json")`). Give every step's file its own
  name, so a check reads the step it means (og-build §1, §3).

## 1. The skills

| Skill | Read it for | Its modules (`reference/<module>.md`) |
|---|---|---|
| **og-core** | the rules every line obeys: TypeScript only, the fixed tick, Results, archetype views, reflection | math, reflect, world, system, rng |
| **og-game** (this one) | the project, scenes, assets and loading, input, 2D, animation, effects, atmosphere, terrain, shaders, physics, performance, looks, light and shadows | game, transform, input, load, stream, render, physics, atmosphere, anim, tilemap, terrain |
| **og-build** | the loop: cook, run with a script, read frames and dumps, hot reload, playtests, `og polish` (§5 there: the one list of what it scores), shipping | commands, probe |
| **og-feel** | how much: hit stop, shake, squash, easing, floating text, the camera, frame rates, sorting, parallax, controller tunables | time, tween, camera, systems |
| **og-ui** | screens and HUDs: templates, widgets, focus, phones, skins and motion | ui |
| **og-taste** | the look: a style from the premise, composition, colour, type, motion, feedback, atmosphere by intent, the done checklist | — (it uses ui and atmosphere) |
| **og-ai** | enemies and NPCs: navigation and minds as data | nav, ai |
| **og-audio** | sound: events, presets, UI sounds, the mix, music, space | audio |
| **og-i18n** | languages: string tables, messages, fonts, line breaking, right to left | i18n |
| **og-platform-sdk** | the platform from inside the game: saves, settings, leaderboards, telemetry, points, AI marks | platform |

Each module's page names its skill, and `reference/index.md` groups the
modules the same way. When two skills seem to touch the same thing, the one
in this table owns it and the other points there.

## 2. The project

| Path | What it is |
|---|---|
| `project.ogt` | The manifest: `name`, `id`, the pinned `engine` version, `entry`, `profile` (`2d` or `3d`), `tick`, `seed`, `scenes`, the first `scene`, `publish` settings |
| `src/main.ts` | The game entry: its default export is `og.game({...})` |
| `assets/` | Source assets: glTF and `.vox` models, PNG/JPEG textures, WAV sounds, TTF/OTF fonts, `.ogt` scenes and looks, and `<file>.ogt` import settings |
| `cooked/` | `og cook` output (`.ogb` blobs and `.cook.ogt`); never committed, the only thing the runtime reads |
| `tests/*.og.ts` | Playtest scripts `og test` runs headless |
| `dist/` | `og build` output |

The engine version in `project.ogt` is a pin. Upgrading it is an explicit
change with its own run, never a side effect. Its `tick`, `seed`, `scenes`
and `scene` are the defaults of the `GameDesc` fields of those names: what
`og.game({...})` leaves unset takes the manifest's value (else 60, 0, none
and `main`). Write each in one place; set in both to different values, the
engine refuses to start and names both.

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
  `font { charset latin chars "开始游戏" }`).
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
  without it one clip `frames` holds every frame). Each frame is centred on
  its own pivot, as in the editor: give the frames one height (or pad them)
  for a character whose feet stay on the ground.
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
budget fails. A chunked sound (og-audio §5) streams on its own and cannot
join a group.

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
  one-tick, immediate drawing (HUDs, debug).
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
- **World UI that must not change with the light** (speech bubbles, name
  tags, markers) takes `og.Sprite.toneMapped: false` (or
  `SpriteDesc.toneMapped`), three.js's `Material.toneMapped`: still placed
  and depth tested in the world, but shown in its colours as given,
  outside auto exposure, bloom, fog and the tone curve, so a cream bubble
  stays cream at dusk instead of blowing out to white. Alpha-blended
  sprites only.
- **Sprite sheets** are `.ogt` files of `type flipbook` next to the PNG
  (grid or named frames, or an Aseprite / TexturePacker JSON `atlas`; clips
  with `fps`, `mode loop | pingPong | once` and `event <frame> "name"`).
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
  `cellCenter` convert positions. Layers scroll by their `parallax`;
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
| `z` | tile layer custom property (int) | the sprite layer offset from og.Tilemap.layer (default: the layer's index among the tile layers) |
| `repeatx`, `repeaty` | tile layer custom properties (bool), or Tiled's own layer fields | the layer repeats along x or y; a property of another name (`repeat`) is not read |
| `parallaxx`, `parallaxy` | Tiled's own layer fields (Layer Properties, Parallax Factor) | the layer's parallax |
| `offsetx`, `offsety` | Tiled's own layer and group fields | the layer's offset, a group's added to its layers' |
| `opacity`, `visible` | Tiled's own layer fields | as named |
| `type` or `class` | an object's | TileObject.kind |
| any other | an object's custom properties | TileObject.properties, every value as text |

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
such as speed pick the motion), `og.IkChain` turns a chain of bones to a
target after the clips, `og.Blendshapes` weighs morph targets, and
`og.AnimationEvent` reports a clip's named frames. Root motion is a
`Skeleton` field. `templates/fx-showcase`'s walker is the pattern.
`og.anim.clip` finds a clip by name, a model's as a flipbook's (§8):

```ts
const anim = must(og.load.animation(world, "models/slime.vox"));
const chase = og.anim.clip(anim, "chase"); // a ClipAsset; 0 when the model has none
world.insert(slime, og.Skeleton, { asset: anim.skeleton });
world.insert(slime, og.AnimationPlayer, { clip: chase, loop: true, stepFps: 12 });
```

A `.vox` with frames (§5) is played the same way: its clips (`frames`, or
the `voxClip` names) swap the frames' models on the frame boundaries.

**Retargeting** — one clip on characters of other proportions and bone
names. Keep shared clips in an animation library (a glTF with a skeleton
and clips, no mesh). At cook time, a character's import settings
(`models/hero.glb.ogt`, `type import`) pull them in:

```text
animation {
  library "anims/humanoid-clips.glb"   # its clips land in hero's #anim, retargeted
  rig "rigs/hero.ogt"                  # optional: how hero's bones are named
}
```

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

**Compression** shrinks a model's clips in the bundle (not in memory: they
are rebuilt at load). In the same `animation` block, `compress true`
quantises rotations to 48 bits and the rest to 16 and drops the keyframes
interpolation rebuilds within `tolerance` (default 0.0001: about 0.1 mm
and 0.01°); a channel 16 bits cannot hold within it stays exact. Turn it on
for a game with many clips; leave it off while a clip's motion is still
being judged.

## 10. Effects

Every effect is off until the game turns it on, as data:

- **Post** — the `og.PostEffects` resource: `ao`, `outline`,
  `depthOfField`, `motionBlur`, `godRays`, `glow`, `lut` (a handle from
  `og.load.lut(world, "luts/look.cube")`), `vignette`,
  `chromaticAberration`, `filmGrain`, `flash`, and a fixed shake of the
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
  `looping false`, one `burst 0 <count>` and a short `duration`; restart
  it by writing the tick to its `og.ParticleSystem.startTick` (moving the
  entity first). A lasting one (embers, motes, rain) keeps the defaults
  `looping true` and a `rate`.
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

**Terrain** (`reference/terrain.md`) is a height field cooked from a
`type terrain` .ogt beside a 16-bit grey PNG (or an EXR in metres):

```text
type terrain
version 1
---
heightmap "island.png"     # 16-bit grey: black and white are `height`'s ends
height -20 120             # metres
size 2048 2048             # metres along x and z, centred on the entity
control "island-control.png"  # optional RGBA: layer 0 in R, 1 in G, 2 in B, 3 in A
heightBlend 0.15           # sharpen the blends by each albedo's alpha (its height)
layer { albedo "textures/grass.png" normal "textures/grass-n.png" tile 6 slope 0 30 }
layer { albedo "textures/rock.png" tile 12 triplanar true slope 28 90 }
layer { albedo "textures/snow.png" altitude 90 1000 altitudeFade 10 }
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
within 60 m is about 36,000 copies. A navmesh .ogt's `terrain "terrain/island.ogt" { step 4 }` bakes
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
body moves by it when it has one. `world.set(e, og.Velocity, ...)` refuses on
an entity without one (a body in `start`, before its first step):
`world.insert` it there, which adds and sets it.

```ts
const hit = og.physics.raycast(world, { origin: eye, direction: { x: 0, y: 0, z: -1 }, maxDistance: 30, filter: { mask: 0b10, exclude: player } });
if (hit.ok && hit.value) { /* hit.value.entity, distance, point, normal */ }
const under = og.physics.shapeCast(world, { shape: { shape: SHAPE_SPHERE, radius: 0.4 }, position: feet, direction: { x: 0, y: -1, z: 0 }, maxDistance: 2 });
const near = og.physics.overlap(world, { shape: { shape: SHAPE_BOX, size: { x: 4, y: 2, z: 4 } }, position: door });   // entities, by id
```

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
  walls, pushes bodies; read `grounded`, `velocity`, `ground`.
- `og.PlatformerController` (2D) — `moveX`, `jump`, `down` in; jump height
  and time to apex, coyote time, jump buffer, wall jumps, one-way layers,
  corner correction as tunables (og-feel §12 has values by genre);
  `og.CharacterEvent` reports jumps and landings for dust and sounds.
  `templates/archetypes/runner`'s courier is one.
- `og.Vehicle` — a raycast car (3D, `throttle`, `steer`, `brake`; wheels
  from `og.physics.vehicleWheels`) or a car on wheel joints (2D).
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
desktop so phones have room. `og run --json` prints `advanceMs`, `og
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
(one block each, fields by name), the UI `skin` and the `antialias`.
Choosing one for a premise is og-taste §1 and §12; this is the mechanism.

```text
type look
version 1
---
name "Harbour"
extends "anime-cel-3d"           # a built-in preset (templates/looks/<slug>.ogt) or another look's id
skin "skins/harbour/skin.ogt"
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
  its skin. Write per-game tweaks in `start` after it, or in the look. A
  system that applies one declares those resources among its writes.
- `extends` flattens at cook time, field by field (a field written wins);
  `assets/looks/<slug>.ogt` of the game's own replaces the preset of that
  slug for every look that extends it. The cook refuses an unknown field,
  a value out of range and an asset not in the bundle, naming the line.
- `og.load.lookInfo` (and `og.render.look`) answer `LookInfo`: `name`,
  `extends`, `resources`, `skin`, `antialias`. A look's `antialias` (tier,
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
  `shadowTintAmount`, `rimIntensity`, `outlineWidth` (with
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

## 16. Light, exposure, shadows and antialiasing

- **Tone mapping and exposure** (`og.RenderSettings`): `toneMapping` is
  `aces` (the default), `agx` (bright saturated lights go to white instead
  of shifting hue: neon, fire, magic) or `none`; `toneMapped false` clamps
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

## 17. Where to look things up

`reference/index.md` (the generated wiki shipped with this skill) maps every
module (grouped by skill, as in §1), type and function to its page, with the IDL text, defaults, ranges
and units of every field.
