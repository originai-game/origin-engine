---
name: og-2d
description: 2D on Origin Engine — sprites as components (anchor, layer, sortY, flash, nine-slice), pixel sprites standing in a 3D set with drop shadows, billboards turned to the camera, world UI kept out of the light, sprite sheets as flipbooks (grids, named frames, Aseprite and TexturePacker atlases, clips and frame events), texture arrays drawn in one batch, and tilemaps (tilesets with collision, animated tiles and blob47 or bitmask16 auto-tiling, hand-written maps, Tiled and LDtk imports, parallax and per-layer collider bits), with every line of the flipbook, tileset and map grammars. Use when a game draws sprites, plays a sprite sheet or builds a level from tiles.
---

A 2D game on Origin Engine draws components, not calls: an `og.Sprite`
on an entity, an `og.Flipbook` playing a sheet, an `og.Tilemap` drawing a
map and giving its walls colliders. The sheets, tilesets and maps are
`.ogt` text beside their PNGs, written by a script and cooked. 2D physics
and the platformer controller are og-physics, 2D skeletons og-anim §6,
2D lighting and particles og-render §1 and §2.

- §1 Sprites, billboards, texture arrays and tilemaps: what each component
  does, with the archetypes that use them.
- §2 Flipbooks: every line of a `type flipbook` sheet, and atlases.
- §3 Tilesets: every line of a `type tileset`, auto-tiling masks.
- §4 Maps and Tiled: a hand-written `type tilemap`, what the cook reads of
  a Tiled map, a map's import settings.

## 1. Sprites, billboards, texture arrays and tilemaps

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
  `pixel-3d-hybrid` look (og-taste §12) wants both.

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
  every line is the grammar of §2).
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
  `colliderLayers` and `colliderMask` (§4's grammar): a layer of grates
  on the bit of the platformer's `oneWayLayers` is one-way while the
  walls on another layer stay solid. Layers scroll by their `parallax`;
  `repeat x` tiles a background (`og api show tilemap`). Every line of a
  tileset and a map, and what the cook reads of a Tiled map, is the
  grammar of §3 and §4.

How many frames, how fast, which layer and how much parallax is og-feel
§9–§11. `templates/archetypes/topdown` (a Zelda-like dungeon: a Tiled map
with auto-tiled walls and collision, a four-direction flipbook hero on a
top-down character controller, an enemy with a hurt flash and knockback,
pickups with floating text, a door to a second room the camera scrolls to),
`templates/archetypes/runner` and `templates/archetypes/match3` use all of
this; `templates/archetypes/roguelike` carves a tilemap at run time from
og.Rng and keeps its fog of war on a layer, and
`templates/archetypes/tower-defense` writes placements to a map's layer.

## 2. Flipbooks: the grammar

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

## 3. Tilesets: the grammar

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

## 4. Maps and Tiled: the grammar

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
