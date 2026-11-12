<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `models` guide: Models as code

The `type model` asset: a prop or stylised character written as text from the engine's own geometry generators, cooked into the same model blob a glTF becomes. Use it when a model has to follow the look's shape language exactly, or when no catalogue or generated asset fits.

The geometry descs are `og.render`'s: `og api find geometry` lists them,
`og api show og.render.boxGeometry` (and the others) gives each field and
default. The shapes and paths are og.Shape's and og.path's.

<a id="guide-models-1"></a>
### `models §1` The file

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

<a id="guide-models-2"></a>
### `models §2` Building one

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

<a id="guide-models-3"></a>
### `models §3` Plants

A `plant "<name>" { ... }` block grows a tree, a shrub or a flower from
numbers and a seed instead of parts: one a model, at its root. The cook
makes its wood and its leaves and petals (in vertex colours, lit from both
faces with light through them), each vertex with what the wind needs, so
the instance sways once `og.Model.sway` is set and og.Wind blows.

- The plant: `kind` (one of the kinds below: the block's defaults are
  that plant's, and every key it names sets only that), `seed`
  (another seed is another individual of the same kind), `growth` (`deciduous`, or `evergreen` for a leader with boughs
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
  `rows` (0 anywhere round the branch; 2 in two rows to its sides, a
  palm's or a fern's frond), `size` (m), `width`, `sizeVariance`, `color` and `color2` (each leaf one
  between the two, linear), `colorVariation`, `translucency`,
  `domeNormals`.
- `flowers { ... }`: `share` of the branch ends that flower, `count` more
  along each branch, `petals`, `size` (a petal's length, m), `width`, `cup`
  (degrees: 0 flat as a daisy, 60 a tulip), `color`, `color2`,
  `centerColor`.

A plant starts from its kind: `plant "palm" { kind palm }` is a whole
palm, and the keys beside `kind` set what differs (without a kind the
defaults are a small broadleaf tree of about 6 m, with leaves and flowers
only when their block is there). A block named inside a kind keeps the
kind's other keys: `leaves { color 0.5 0.3 0.02 }` is the same tree in
another colour.

| `kind` | The plant | Height, triangles | What gives its shape |
|---|---|---|---|
| `oak` | a broadleaf tree with a round crown | 13.6 m, 37 000 | `levels 3`, a trunk forking into 6, leaves `count 60` |
| `birch` | a slender tree, pale bark, small leaves | 9.1 m, 17 000 | a thin trunk `radius 0.11`, short limbs at `angle 38`, drooping a little |
| `willow` | a weeping tree | 7.6 m, 40 000 | `force 0 1 0 -0.035` on long thin `branch3` shoots, leaves in `rows 2` |
| `pine` | a conifer | 10.1 m, 48 000 | `growth evergreen`, 60 boughs at `angle 105`, dark narrow leaves |
| `palm` | a palm | 10.2 m, 4 300 | `levels 1`: 17 fronds from `start 0.97` of a bare trunk, leaves in `rows 2` |
| `bamboo` | one cane (stand several close together) | 7.7 m, 3 000 | a green trunk `radius 0.035 taper 0.35`, short twigs |
| `cactus` | a saguaro, flowers on its arms | 3.8 m, 1 300 | thick arms `taper 0.12` turned up by `force 0 1 0 0.12`, no leaves |
| `shrub` | a flowering bush | 2.7 m, 20 000 | a trunk of `length 0.05` that branches at the ground, flowers `share 0.8` |
| `fern` | a fern | 0.9 m, 2 000 | 11 fronds from the ground at `angle 34`, leaves in `rows 2` |
| `reed` | a tuft of reeds or tall grass with plumes | 1.7 m, 700 | 7 stems at `angle 7`, long leaves `width 0.05`, closed flowers `cup 82` |
| `daisy` | a flower with a rosette at its foot | 0.4 m, 110 | `levels 0`, a green stem, flowers `petals 14 cup 8` |
| `tulip` | a cupped flower | 0.4 m, 70 | `levels 0`, flowers `petals 6 cup 68`, three upright leaves |

Colours and seeds make the variety, a file each: `color` and `color2` on
leaves and flowers give autumn trees, pale grasses and flowers of any
colour from the same shape (`kind tulip` with `flowers { color 1 0.7 0
}`, `kind birch` with `leaves { color 0.5 0.3 0.02 color2 0.6 0.14 0.02
}`), three or four seeds of one kind stop a wood from repeating, and
`trunk { length }`, a level's `children` or the leaves' `size` make a
young, an old or a sparse one. A plant no kind is near (a vine, a
mushroom's cap) is a model of parts or an imported one.

A plant's triangles are mostly its leaves (`children` multiplied down
the levels, times `count`, four triangles a blade). As written above the
oak is 37 000 triangles (34 000 of them leaves), the pine 48 000, the
shrub 20 000, the daisy 114. With distance the cook thins the leaves
instead of simplifying them: two more levels keep a half and a quarter
of the leaves and petals, each kept one made larger so the crown covers
as much (the oak's leaves are 34 000, 17 000 and 8 500 triangles), and a
plant of fewer than 16 leaves keeps them all. A copy takes a level once
what it moved the leaves by is under 2 pixels, so the copies near the
camera are the full plant: one that stands close in tens or hundreds
takes a lower `count` with a larger `size` first, then fewer `children`.
The cook refuses a plant past 20 000 branches or 120 000 leaves and
petals.

A wood is plants scattered whole (og.TerrainScatter's `model`,
`og api show "world §3"`). Far copies draw as one card when the plant is cooked
with an impostor: import settings beside the model text, named after it
with one more `.ogt` (`oak.ogt.ogt` for `oak.ogt`), holding `type
import`, `version 1` and `model { impostor true }`. The card is lit as
the leaves are (the same light, the plant's own shade baked in), but
it is a picture 128 texels across the plant (`impostorSize`): nearer
than where a texel covers a pixel it reads soft beside the leaves. Set
impostorDistance to at least the plant's radius times the screen's
height in pixels, divided by 60 (a 50 degree view; 12 radii at 720
rows, 18 at 1080; the radius is that of the sphere around the whole
plant): 110 m for an oak of 9 m at 720 rows, 28 m for a shrub of 2.3 m.
A larger `impostorSize` brings that nearer in proportion and multiplies
the atlas by its square.

og.Wind's `strength` is lean per metre of height: 0.05 to 0.1 moves a
10 m tree as a breeze does, and what suits grass (0.5 and up) lays a tree
flat. Give trees their own lower `sway` when one wind blows over both.

<a id="guide-models-4"></a>
### `models §4` Pitfalls

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
