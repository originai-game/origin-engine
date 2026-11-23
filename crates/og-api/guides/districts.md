# Districts: towns from rules

The `type district` asset: a whole area (streets, pavements, buildings and roadside props) laid out by the cook from rules and a seed, instead of placing models one at a time. A district names building rules (`type grammar`) and a material library (`type materials`); the cook turns it into a scene of ordinary models, colliders and streaming cells, so it loads, streams and draws like any scene. Nothing is generated at runtime.

The scene it becomes is placed and streamed as world §5 describes; its
models are glTF models with levels of detail (models §1 for the material
fields). Every number is the game's: the cook ships no town, no style and
no default building.

## 1. The three files

All three live under `assets/`. Only the district is placed in a game; the
grammars and the library it names are part of it.

A grammar builds one building on one lot. It starts from the lot's
footprint and hands faces from rule to rule until each ends in a `label`
(the part is made and labelled), an `insert` (a model is fitted to the
face) or `drop`:

```text
type grammar
version 1
---
param height 12
param floor 3.2
start Lot
rule Lot {
  extrude height {
    sides Facade
    top Roof
  }
}
rule Facade {
  split y {
    part 4 Ground
    repeat floor Floor
    part 0.4 Cornice
  }
}
rule Floor {
  split x {
    part 0.5 Wall
    repeat 2.8 Bay
    part 0.5 Wall
  }
}
rule Bay {
  split x {
    fill 1 Wall
    part 1.2 Window
    fill 1 Wall
  }
}
rule Window {
  split y {
    part 0.9 Wall
    part 1.6 Glass
    fill 1 Wall
  }
}
rule Glass {
  detail 1
  recess 0.12
  label glass
}
rule Ground {
  label shopfront
}
rule Cornice {
  label trim
}
rule Wall {
  label wall
}
rule Roof {
  roof hip 35 {
    overhang 0.3
    slope Slate
  }
}
rule Slate {
  label roof
}
```

- `extrude` names the prism's faces: `front` (the lot's street edge),
  `back`, `side`, `sides` (every side not named), `top`, `bottom`.
- `split x` cuts across a face, `split y` up it: `part` is exactly its
  size, `fill` stretches to what the parts leave, `repeat` fits as many
  copies as fit, stretched to fill.
- `inset <width> <border> <inner>`, `pick { option <weight> <rule> }` and
  `case { wider|taller <n> <rule>; param <name> <n> <rule>; otherwise
  <rule> }` choose and cut; `recess <depth> [<jamb rule>]` pushes a face
  in and makes its jambs (labelled `jamb` without a rule); `roof flat`,
  `roof gable <deg>` and `roof hip <deg>` build roofs.
- `detail <n>` leaves what the rule reaches out of the model's levels of
  detail from level n on: glass recesses, sills and mouldings at 1 or 2,
  the walls and roof at 0.
- A param named `height` takes the height the district draws for the lot.

A material library gives each label its materials, each a `type model`
material (models §1) with its own fields:

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
  rule "grammars/terrace.ogt" {
    weight 3
  }
  rule "grammars/corner-shop.ogt" {
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

## 4. Pitfalls

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
