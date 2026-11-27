# Districts: towns from rules

The `type district` asset: a whole area (streets, pavements, buildings and roadside props) laid out by the cook from rules and a seed, instead of placing models one at a time. A district names checked TS building rules (`file.grammar`) and a material library (`type materials`); the cook turns it into a scene of ordinary models, colliders and streaming cells, so it loads, streams and draws like any scene. Nothing is generated at runtime.

The scene it becomes is placed and streamed as world §5 describes; its
models are glTF models with levels of detail (models §1 for the material
fields). Every number is the game's: the cook ships no town, no style and
no default building.

## 1. The three files

Grammars export `file.grammar` from `content/**/*.ts`; districts and material
libraries still live under `assets/` until their domain cutovers. Only the
district is placed in a game; grammars and the library are cook inputs.
District `buildings.rule` names a grammar's canonical content ID, such as
`base/grammars/terrace`, not a physical path. No grammar OGT reader remains.

A grammar builds one building on one lot. It starts from the lot's
footprint and hands faces from rule to rule until each ends in a `label`
(the part is made and labelled), an `insert` (a model is fitted to the
face) or `drop`:

```ts
import { file } from "@origingame/engine/author";

export default file.grammar({
  params: { height: 12, floor: 3.2 },
  start: "Lot",
  rules: {
    Lot: [{ op: "extrude", depth: "height", faces: { sides: "Facade", top: "Roof" } }],
    Facade: [{ op: "split", axis: "y", parts: [
      { size: "abs", length: 4, rule: "Ground" },
      { size: "repeat", length: "floor", rule: "Floor" },
      { size: "abs", length: 0.4, rule: "Cornice" },
    ] }],
    Floor: [{ op: "split", axis: "x", parts: [
      { size: "abs", length: 0.5, rule: "Wall" },
      { size: "repeat", length: 2.8, rule: "Bay" },
      { size: "abs", length: 0.5, rule: "Wall" },
    ] }],
    Bay: [{ op: "split", axis: "x", parts: [
      { size: "fill", length: 1, rule: "Wall" },
      { size: "abs", length: 1.2, rule: "Window" },
      { size: "fill", length: 1, rule: "Wall" },
    ] }],
    Window: [{ op: "split", axis: "y", parts: [
      { size: "abs", length: 0.9, rule: "Wall" },
      { size: "abs", length: 1.6, rule: "Glass" },
      { size: "fill", length: 1, rule: "Wall" },
    ] }],
    Glass: [{ op: "recess", depth: 0.12 }, { op: "label", label: "glass" }],
    Ground: [{ op: "split", axis: "x", parts: [
      { size: "abs", length: 0.4, rule: "Wall" },
      { size: "fill", length: 1, rule: "Shop" },
      { size: "abs", length: 1.35, rule: "Entrance" },
      { size: "abs", length: 0.4, rule: "Wall" },
    ] }],
    Shop: [{ op: "split", axis: "y", parts: [
      { size: "abs", length: 0.45, rule: "Plinth" },
      { size: "fill", length: 1, rule: "ShopWindow" },
      { size: "abs", length: 0.45, rule: "ShopFrame" },
    ] }],
    ShopWindow: [{ op: "inset", width: 0.1, border: "ShopFrame", inner: "ShopGlass" }],
    ShopGlass: [{ op: "recess", depth: 0.16, jamb: "ShopFrame" }, { op: "label", label: "shopglass" }],
    Entrance: [{ op: "split", axis: "y", parts: [
      { size: "abs", length: 0.12, rule: "Plinth" },
      { size: "abs", length: 2.6, rule: "DoorFrame" },
      { size: "fill", length: 1, rule: "ShopFrame" },
    ] }],
    DoorFrame: [{ op: "inset", width: 0.1, border: "ShopFrame", inner: "Door" }],
    Door: [{ op: "recess", depth: 0.2, jamb: "ShopFrame" }, { op: "label", label: "door" }],
    ShopFrame: [{ op: "label", label: "shopfront" }],
    Plinth: [{ op: "extrude", depth: 0.06, faces: { top: "PlinthEdge", sides: "PlinthEdge" } }],
    PlinthEdge: [{ op: "label", label: "plinth" }],
    Cornice: [{ op: "label", label: "trim" }],
    Wall: [{ op: "label", label: "wall" }],
    Roof: [{ op: "roof", roof: { kind: "hip", angle: 35 }, overhang: 0.3, slope: "Slate" }],
    Slate: [{ op: "label", label: "roof" }],
  },
});
```

- `extrude` names the prism's faces: `front` (the lot's street edge),
  `back`, `side`, `sides` (every side not named), `top`, `bottom`.
- A split's `axis: "x"` cuts across a face, `"y"` up it: `size: "abs"` is exactly its
  size, `fill` stretches to what the parts leave, `repeat` fits as many
  copies as fit, stretched to fill.
- `inset` names `width`, `border`, `inner`; `pick` has ordered
  `options: [{weight, rule}]`; `case` has ordered
  `arms: [{condition, rule}]` and optional `otherwise`. Conditions select
  `kind: "wider" | "taller" | "param"` with a `value` (and `name` for param).
  `recess` names `depth` and optional `jamb` (labelled `jamb` without a rule).
  A roof selects `roof: {kind: "flat"}` or gable/hip with `angle` in degrees.
  Inserted models use `model: ref("module/content-id")`.
- `detail` with `level: n` leaves what the rule reaches out of the model's levels of
  detail from level n on: added sills and mouldings at 1 or 2,
  the walls and roof at 0.
- A param named `height` takes the height the district draws for the lot.

The facade keeps its openings at every LOD: marking a pane or the whole
ground storey `detail 1` would leave a hole at a distance. Its door, glazing,
frame and plinth have separate labels so changing the wall material does
not turn an entrance back into a blank wall. Cover `shopglass`, `door`,
`shopfront`, `plinth`, `trim` and `roof` in the library as well. For a
mixed street, a `pick` in `Ground` can choose this shopfront or a second
composition with a narrow entrance and raised residential windows; put
that choice on the street's front face, not every party wall.

A material library gives each label its materials, each sharing the native
surface fields of `file.model` materials (models §1):

```text
type materials
version 1
---
material "brick" {
  label wall
  label jamb
  weight 3
  tile 1.2 0.9
  vary {
    value 0.12
    warmth 0.06
  }
  map "textures/brick.png"
  normalMap "textures/brick-normal.png"
  roughness 0.85
}
material "render" {
  label wall
  label jamb
  color 0.78 0.74 0.66
  roughness 0.9
}
material "glass" {
  label glass
  color 0.02 0.03 0.04
  roughness 0.05
  metalness 0.2
}
```

- `label`: what it covers (one or more). `weight`: how often a building
  draws it among the materials of a label. `tile`: the metres one repeat
  of its maps covers, across and up. `vary`: how far each building's
  brightness (`value`) and warm or cool cast (`warmth`) move, 0 to 0.5.
- Each building draws one material per label from its own seed. Labels
  covered by the same materials draw the same one, so give a wall and its
  jambs the same list and the brick runs into the reveals.
- The streets draw labels too: `road`, `pavement`, `kerb` and `ground`
  (the yard behind the lots). Cover them in the library.

The district:

```text
type district
version 1
---
seed 7
region {
  point 0 0
  point 400 0
  point 400 300
  point 0 300
}
streets {
  grid 100 80
  turn 12
  jitter 0.1
  width 10
  sidewalk 3
  kerb 0.15
}
lots {
  width 6 14
  depth 14 24
}
buildings {
  density 0.9
  height 9 24
  heightNoise 0.01 0.6
  rule "base/grammars/terrace" {
    weight 3
  }
  rule "base/grammars/corner-shop" {
    street "High Street"
  }
}
prop "lamp" {
  model "models/lamp.glb"
  every 22
  jitter 1
  kerb 0.5
  corners true
  collider capsule 0.12 4
}
materials "materials/london.ogt"
cells {
  size 64
  load 160
}
```

- `region` is an outline the street grid and the named streets cut; or
  `along { point ...; width; sidewalk; depth; name }` lines both sides of
  one street, `depth` metres deep, for a single road.
- `streets`: the grid's spacing along its x and z, its turn in degrees,
  how far each line moves (a share of the spacing, 0 to 0.45), the
  roadway's width, the pavement on each side and the kerb's height. Named
  streets (`street "<name>" { point ...; width; sidewalk }`) cut the
  blocks too, and rules and props can keep to them by name.
- `lots`: each lot's width along its street and the depth of the ring of
  lots round a block, least and greatest. Every lot fronts a street.
- `buildings`: the share of lots built on, the height range, and
  `heightNoise <frequency per metre> <share>`, the share of each height
  that follows smooth noise over the town (tall streets and low ones)
  rather than the lot's own draw. `rule` lines pick a grammar by weight,
  optionally only along one street.
- `prop`: a model along every pavement (or one `street`), `every` metres,
  `kerb` metres from the kerb, moved up to `jitter`, one more at each
  corner with `corners true`, colliding as `box <w> <h> <d>` or
  `capsule <radius> <height>`.

## 2. What the cook makes

- One scene with the district's id. A building collides as the box round
  its lot up to its roof; each prop is an entity placing its model with
  its collider, so copies draw instanced. Models a grammar `insert`s are
  placed the same way.
- Without `cells`, every building, road and pavement is in one model.
  With `cells { size; load }`, each square of `size` metres that holds
  anything is an og.StreamCell with its own sub-scene and cell group
  (world §5), loaded within `load` metres; an `hlod` block in the district
  is given to every cell (world §5, Distant models as proxies).
- A cell's buildings, roads and pavements are one model, a primitive per
  material, so a cell draws in a handful of calls. `merge false` keeps a
  model per building instead (to hide, swap or pick one building).
- Each model has a level of detail per `detail` level its grammars use.
- Every entity is named under the district's asset id, as names are
  unique in the whole world: `districts/town.ogt/building-12`,
  `districts/town.ogt/lamp-3` and its `/collider`, a cell's
  `districts/town.ogt.cell-2-1` and its merged `/model`.
- The same district, grammars, library and seed give the same bytes on
  every host; another seed gives another town with the same rules.

## 3. Working on one

1. Write one grammar and a library of flat colours, and a small `along`
   district of one street. Cook and look at it from the street before
   adding maps: proportions (floor heights, bay widths, the shopfront)
   carry a street more than textures do.
2. Add the props, then the second and third grammar, weighted.
3. Move to a `region` with a grid, and add `cells` once it is more than a
   few hundred metres across. Walk it with the profiler on (perf guide):
   draw calls per cell are the materials it uses.
4. Change the seed to see other towns from the same rules; keep the seed
   the game ships in the district, not in code.

## 4. A library from generated images

Keep image generation at authoring time. A library may describe what each
surface looks like; the engine turns those descriptions into requests and
processes the returned images, but calls no image service:

```text
material "stock-brick" {
  label wall
  label jamb
  tile 2 1
  brief "yellow London stock brick, irregular fired faces, pale lime mortar"
  map "textures/stock.png"
  relief -0.015
  normalMap "textures/stock-n.png"
  roughness 1
  roughnessRange 0.75 0.95
  metalnessMap "textures/stock-mr.png"
  occlusion 0.5 3
  aoMap "textures/stock-ao.png"
}
```

1. Run `og materials brief assets/materials/london.ogt --json`. Each
   request has the project's look, palette and accepted style, an image
   size and tile metres, its `requestHash` and a `source` asset id. Pass
   the returned `brief` to the authoring platform's image tool, not a
   second hand-written prompt. It asks for base colour without baked
   light or perspective. A facade module adds `facade true`: its left and
   right edges repeat, but its ground and roof must not meet. The cook
   anchors its bottom row at the building base instead of varying its
   vertical phase. Set its tile height to the facade height: a module is
   not automatically stretched to fit buildings of different heights.
2. Save each returned image at `assets/<source>`, for example
   `assets/textures/stock.source.png`. Requests are stable until the look,
   style or subject changes; record the hash with the generated asset.
3. Run `og materials make assets/materials/london.ogt --json`. It
   resamples each side down to a power of two (at least 8 texels), removes
   the smooth boundary discontinuity with periodic-plus-smooth
   decomposition, then writes the named PNG maps. A facade is processed
   across only. This reduces edge steps; it cannot align brick courses or
   repair an incorrectly drawn window. Inspect several repeats before
   accepting the asset.
4. Inspect `seamBefore`, `seamAfter` and `interior` (mean neighbouring
   texel step), dimensions and texels per metre. These are facts, not a
   quality score. Missing sources are listed with exit 1, invalid input
   exits 2. The cache in `.og/materials/` keys source bytes and processing
   options, and verifies output hashes before reusing a result. Editing a
   source or option, or damaging an output, causes it to be remade.
5. Keep the accepted sources and maps in the platform's asset store. For
   an engine fixture use `bun scripts/assets.ts pin <fixture-dir> <paths...>`
   and `push <fixture-dir>`, recording the request hash in the manifest's
   `source`. Commit the manifest and ignore rules, not the image binaries.
   The ordinary cook packs the maps; nothing generates images at runtime.

Derived maps are optional, authored approximations, not physical
measurements inferred by AI. `relief` gives metres from black to white in
linear luminance: negative relief sinks pale mortar. It needs a
`normalMap` destination. `roughnessRange` gives linear roughness at black
and white (either order, both 0 to 1), written to `metalnessMap`'s green
channel; blue is 1, multiplied by the material's `metalness` as in glTF.
`occlusion` gives cavity darkening strength (0 to 1) and a whole sampling
radius (1 to 64 texels), written to `aoMap`, and needs nonzero relief.
Omit these derivations to keep independently authored PBR maps. Only
materials with a `brief` are processed; every generated destination is a
distinct PNG under `assets/`, and the tool does not follow asset symlinks.
`og model bake` still transfers a model's own geometry and materials; it
does not call an image model or replace this surface-image workflow.

## 5. Aging and tiling

Keep the library's per-building colour and phase variation for differences
between neighbours. For aging within a wall, add the authored surface
fields to its material:

```text
  grime 0.4
  grimeColor 0.07 0.05 0.025
  grimeHeight 1.5
  stains 0.3
```

Grime starts at each building's base, even when buildings merge in one
cell. A library's tile height changes the image repeat, not the dirt's
height. All its PBR maps keep the same transform. Dry stains stay fixed
in world space with no weather simulation; simulated wet streaks remain
separate and continue to follow the weather. Neither generates a texture.
On a manually made model, provide metric UVs (v = negative height from the
base) before map transforms if it needs ground-up grime.

Age the surfaces that actually reach the pavement. If brick starts above
a four-metre shopfront, a 1.5 m grime band on `wall` cannot touch it. Give
the ground-floor frames, door and plinth their own aging fields instead
of raising the wall's dirt height to cover upper storeys. Masonry piers
between the shop windows can use the wall material down to the plinth.

For unstructured plaster, gravel or mottled stone, `stochastic 1` breaks
up ordinary 2D texture repetition using shared, unrotated patch offsets.
Use matching map transforms and resolutions to keep the PBR maps aligned.
Leave this off on bricks whose courses must line up, facade modules,
windows, text, atlases and deliberately placed damage: random patches
cannot preserve their structure. Array maps are not randomized. The
histogram transforms are made once when the material's maps change, not
each frame; no image model runs. The transform uses 12 KiB per randomized
material, without another image copy. Under minification it fades back to
ordinary filtered mip sampling to avoid shifting the distant colour
distribution; this is not a guarantee of repetition-free distant walls.

## 6. Streets on terrain

When a district crosses a height field, give it the terrain asset to
carve at cook time:

```text
terrain "base/terrain/land" {
  depth 0.25
}
```

Place the district and terrain in the same local coordinate space. Roads
are at local y = 0; the cook lowers samples under their footprint using
the same underside projection as TerrainStamp. `depth` is clearance under
the road, in terrain metres (omitted: zero). Footprint-edge samples meet
the road itself. It cuts only: ground already below that surface stays
below it, so prepare or fill low terrain separately when a road needs
support. Neither source heightmaps nor terrain outside the footprint are
rewritten.

The carve includes the pavement strip but not block interiors, so it is
not a way to level an entire building site. Buildings still start at
the pavement's height; prepare building sites separately on hilly land.
This does not drape the district or choose a road grade. Both region
grids and `along` paths use their generated footprints, including turns.

Place and stream the terrain as usual; naming it here changes its cooked
height field, not its scene placement. Its tiles, LODs, height sampling,
terrain collisions and navmeshes baked from it read the carved surface.
Several districts may carve one terrain; overlapping streets keep the
lowest surface. Editing or removing a district's carve invalidates the
terrain and its nav bakes without changing the original asset files.

## 7. Pitfalls

- A label no material covers fails the cook, naming the label and the
  grammar; a typo in a grammar's `label` is the usual cause.
- A building whose walls and jambs show different bricks has them in
  different material lists; give them the same.
- `roof hip` needs a convex face, and `roof gable` on a face that is not
  a rectangle builds a hip: a lot or a split part that is not convex fails
  the cook, naming the lot. Lots are cut from convex blocks; an `inset` or
  a `case wider` arm with a flat roof keeps odd shapes safe.
- Faces have no thickness: a `recess` makes the depth a window reads by.
  Without one, glass sits flush on the wall.
- Lots stand on the pavement, the kerb's height above the road; a grammar
  starts from there.
