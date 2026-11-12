# Models as code

The `type model` asset: a prop or stylised character written as text from the engine's own geometry generators, cooked into the same model blob a glTF becomes. Use it when a model has to follow the look's shape language exactly, or when no catalogue or generated asset fits.

The geometry descs are `og.render`'s: `og api find geometry` lists them,
`og api show og.render.boxGeometry` (and the others) gives each field and
default. The shapes and paths are og.Shape's and og.path's.

## 1. The file

An `.ogt` under `assets/` (for example `assets/models/crate.ogt`) that starts `type model`, `version 1`,
`---`, then these blocks (a plant in §3):

- `material "<name>" { ... }`: glTF metallic-roughness, by three.js
  names: `color` (linear, as `baseColorFactor`), `opacity`, `metalness`,
  `roughness`, `emissive`, `emissiveIntensity`, `doubleSided`,
  `alphaTest`, `map` and `normalMap` (image files beside the model),
  `normalScale`, `transmission`, `ior`, `thickness`, `clearcoat`,
  `clearcoatRoughness`, and a leaf's `translucency` and `domeNormals`.
  Unit fields are 0 to 1; an unknown key is an error.
- `pivot "<name>" { position ...; parent "<pivot>" }`: a named node that
  animation and code find by name. A part hangs from its `pivot` (default:
  the model's root).
- `part "<name>" { material ...; pivot ...; <geometry> { ... }; at { ... } }`:
  one geometry block named by its `og.render` call (`boxGeometry`,
  `cylinderGeometry`, `coneGeometry`, `capsuleGeometry`, `sphereGeometry`,
  `torusGeometry`, `latheGeometry`, `shapeGeometry`, `extrudeGeometry`,
  `sweepGeometry`, the polyhedra, `csgGeometry` with operands `a` and `b`,
  and `mergeGeometries` of nested `part` blocks), and its placement
  (`position`, `quaternion`, `scale`, `mirror`).

A three.js outline pastes into og.Shape as it is. Make shapes in start or
a command. A geometry's `shape` block is og.Shape's calls, one a line, with
`hole { }` blocks inside it:
`extrudeGeometry { shape { absarc 0 0 1 0 6.283185307179586 false } depth 0.2 }`.
Several `at { }` blocks place one mesh drawn instanced. `og cook` builds
the file into the model blob a glTF cooks to (levels of detail, raycast
BVH, nodes) and refuses it past project.ogt's triangle and vertex budgets;
`og dev` re-cooks it on an edit.

A scene's `model` line names the file, and `og.load.gltf` loads its blob
like any glTF. The look presets carry examples under
`templates/looks/<slug>/models/`, such as paper-craft's `tree.ogt` and
`hero.ogt`.

## 2. Building one

1. Start from the look: its `ArtShape` gives the face budgets
   (`propFaces`, `characterFaces`), the normals (smooth, flat, by angle)
   and the bevel; its palette gives the colours. A model over budget is
   built by the cook at fewer segments (every count scaled by one factor,
   down to a 5-sided round), so write the counts the style wants instead
   of dense ones.
2. Block the silhouette with three to six primitives at the right
   proportions before any detail. Check it from the front and the side
   (`view:<entity>:front`, `view:<entity>:side` on the command channel)
   before adding parts.
3. Repeat a part with `at` blocks, not copies: one mesh under several
   nodes draws instanced. `mirror` (or a negative scale) bakes a mirrored
   copy with its triangles turned, so symmetric halves cost one mesh each.
4. Put moving parts under pivots placed at their hinge (a lid's back edge,
   a wheel's axle). A pivot at the part's centre rotates it through itself.
5. Use `csgGeometry` for cut-outs (windows, sockets) only where an
   extrusion with a `hole` cannot do it: CSG adds faces along every cut.
6. Review with `turntable:<entity>:<n>` and `og compare` (`art-review §5` gives
   the thresholds), then fix the largest error first.

## 3. Plants

A `plant "<name>" { ... }` block grows a tree, a shrub or a flower from
numbers and a seed instead of parts: one a model, at its root. The cook
makes its wood and its leaves and petals (in vertex colours, lit from both
faces with light through them), each vertex with what the wind needs, so
the instance sways once `og.Model.sway` is set and og.Wind blows.

- The plant: `seed` (another seed is another individual of the same
  kind), `growth` (`deciduous`, or `evergreen` for a leader with boughs
  shortening to its top), `levels` (branch levels past the trunk, 0 to 3),
  `force x y z strength` (the limbs bend towards the direction; a negative
  strength droops them), `barkColor` or `material`.
- A level, `trunk` and `branch1` to `branch3`: `length` (m), `radius` (m
  for the trunk, a share of its parent's for a branch), `sections` and
  `segments` (rings along it, sides round it), `taper`, `gnarliness` (how
  far each section wanders), `twist`, `children` (the branches each one
  grows), and for a branch `angle` from its parent (degrees) and `start`
  (where along its parent the first may grow).
- `leaves { ... }`: `shape` (`blade`, a pointed leaf without a texture;
  `card` or `cross`, quads for a leaf texture through `material`), `count`
  along each last-level branch, `start` and `end` (the part of the branch
  they grow along), `angle`, `facing` (0 as they grow, 1 flat to the sky),
  `size` (m), `width`, `sizeVariance`, `color` and `color2` (each leaf one
  between the two, linear), `colorVariation`, `translucency`,
  `domeNormals`.
- `flowers { ... }`: `share` of the branch ends that flower, `count` more
  along each branch, `petals`, `size` (a petal's length, m), `width`, `cup`
  (degrees: 0 flat as a daisy, 60 a tulip), `color`, `color2`,
  `centerColor`.

Every key has a default (a small broadleaf tree of about 6 m), so start
from the kind nearest to the plant and change what differs:

| Kind | What makes it |
|---|---|
| Broadleaf tree (oak) | `levels 3`; trunk `length 7.4 radius 0.28 children 6`; branch1 `length 2.2 angle 54 start 0.49 children 4`; branch2 `length 2.5 angle 58 children 3`; branch3 `length 1.4 angle 32`; leaves `count 60 size 0.3 width 0.55` |
| Conifer (pine) | `growth evergreen`, `levels 2`, `force 0 1 0 -0.004`; trunk `length 10 radius 0.21 gnarliness 0.012 children 60`; branch1 `length 4.4 radius 0.36 angle 105 start 0.22 children 9`; branch2 `length 1.5 angle 58`; leaves `count 16 size 0.24 width 0.2 facing 0.6`, dark `color 0.03 0.11 0.045` |
| Flowering shrub | `levels 3`; trunk `length 0.05 radius 0.11 children 7` (it branches at the ground); branch1 `length 1.5 angle 21.5 start 0.53`; leaves `count 30 size 0.15 start 0`; flowers `share 0.8 count 1 size 0.085 cup 25` |
| Flower (daisy) | `levels 0`, `force 0 1 0 0`, `barkColor 0.1 0.25 0.05` (a green stem); trunk `length 0.4 radius 0.006 gnarliness 0.02`; leaves `count 7 start 0 end 0.3 facing 1 size 0.09` (a rosette at its foot); flowers `petals 14 size 0.055 width 0.3 cup 8` |

Colours make the variety: `color` and `color2` on leaves and flowers give
autumn trees, pale grasses and flowers of any colour from the same shape,
and three or four seeds of one kind stop a wood from repeating.

A plant's triangles are mostly its leaves (`children` multiplied down
the levels, times `count`, four triangles a blade), and leaves are never
simplified with distance. As written above the oak is 37 000 triangles
(34 000 of them leaves), the pine 48 000, the shrub 20 000, the daisy
114. A plant that stands in tens or hundreds takes a lower `count` with
a larger `size` first, then fewer `children`; the cook refuses one past
20 000 branches or 120 000 leaves and petals.

og.Wind's `strength` is lean per metre of height: 0.05 to 0.1 moves a
10 m tree as a breeze does, and what suits grass (0.5 and up) lays a tree
flat. Give trees their own lower `sway` when one wind blows over both.

## 4. Pitfalls

- Colours are linear. An sRGB colour picked from a reference reads too
  light: convert it (or take the palette's value from the look).
- An `extrudeGeometry` with a bevel grows by `bevelSize` on every side:
  shrink the shape to keep the outer size.
- A `type model` has no skin. A character that must animate as a humanoid
  is a rigged glTF (the cook's auto-rig, `og api find auto rig`), or a
  model whose limbs hang from pivots and move by code or clips.
- The cook's mesh LOD chain and raycast BVH treat it as any glTF: a
  `type model` needs no export step.
- A model grows one plant: the wind bends a model as one tree about its
  root, so two plants in one file are two files.
- Leaves that name a `material` take `translucency` and `domeNormals` from
  that material block; without either the leaves draw one face, as bark.
