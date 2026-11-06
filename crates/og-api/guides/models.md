# Models as code

The `type model` asset: a prop or stylised character written as text from the engine's own geometry generators, cooked into the same model blob a glTF becomes. Use it when a model has to follow the look's shape language exactly, or when no catalogue or generated asset fits.

The geometry descs are `og.render`'s: `og api find geometry` lists them,
`og api show og.render.boxGeometry` (and the others) gives each field and
default. The shapes and paths are og.Shape's and og.path's.

## 1. The file

An `.ogt` under `assets/` (for example `assets/models/crate.ogt`) that starts `type model`, `version 1`,
`---`, then three kinds of block:

- `material "<name>" { ... }`: glTF metallic-roughness, by three.js
  names: `color` (linear, as `baseColorFactor`), `opacity`, `metalness`,
  `roughness`, `emissive`, `emissiveIntensity`, `doubleSided`,
  `alphaTest`, `map` and `normalMap` (image files beside the model),
  `normalScale`, `transmission`, `ior`, `thickness`, `clearcoat`,
  `clearcoatRoughness`. Unit fields are 0 to 1; an unknown key is an error.
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

## 3. Pitfalls

- Colours are linear. An sRGB colour picked from a reference reads too
  light: convert it (or take the palette's value from the look).
- An `extrudeGeometry` with a bevel grows by `bevelSize` on every side:
  shrink the shape to keep the outer size.
- A `type model` has no skin. A character that must animate as a humanoid
  is a rigged glTF (the cook's auto-rig, `og api find auto rig`), or a
  model whose limbs hang from pivots and move by code or clips.
- The cook's mesh LOD chain and raycast BVH treat it as any glTF: a
  `type model` needs no export step.
