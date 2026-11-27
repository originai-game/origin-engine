# Models as code

Author procedural models as checked TypeScript under `content/`, with one default `file.model(...)` declaration per entry. The cook builds the same model blob as a glTF import; the game loads its canonical content ID. There is no model OGT parser, sibling import file or TS-to-text bridge. Kernel programs use `file["model-program"]` instead (`model`).

Use procedural models when the silhouette needs to follow the look's shape language or when an imported model does not fit. `og api show file:model` describes the complete schema; geometry descs use the runtime's three.js vocabulary and shared geometry kernels, not another set of generators.

## 1. The file

```ts
import {file} from '@origingame/engine/author';
export default file.model({kind:'geometry',desc:{items:[
  {name:'wood',input:{kind:'material',desc:{color:[0.3,0.16,0.07],roughness:0.8}}},
  {name:'hinge',input:{kind:'pivot',desc:{transform:{position:[0,1,-0.5]}}}},
  {name:'lid',input:{kind:'part',desc:{
    material:'wood',pivot:'hinge',
    geometry:{kind:'boxGeometry',desc:{width:1,height:0.1,depth:1}},
    placements:[{transform:{position:[0,0,0.5]}}]
  }}}
]}});
```

Ordered items declare named materials, pivots, parts and a plant. Declare a dependency before its consumer. A pivot is a parent-first named node; parts attach to it and animation channels address their stable node names. Repeated placements instance one mesh; reflections bake triangle winding and tangent handedness correctly. Give a model under `content/models/chest.ts` the ID `base/models/chest` in a scene or `og.load.gltf` call.

Generate mesh arrays directly in TS, in the declaration or an imported `lib/**/*.ts` helper. Put its result in a `bufferGeometry` desc. Use `input.bytes`/`input.text` for explicit raw inputs and `rng(seed)` for deterministic randomness. No `bufferGeometry.generate` wrapper or script filename is needed. Explicit normals, UVs, colours and tangent frames survive import and placed merges; topology-changing CSG regenerates frames when required. See `generating` for an executable multi-file recipe.

Shapes are structured path commands, including holes, consumed by the same outline and extrusion kernels as the runtime. A tube's sampled points or curve path make a curved tube; this differs from the model program's hollow straight tube solid. `modelGeometry` uses a canonical content ref and optional node name, never a filename guessed beside the declaration. Cycles are errors, and selected nodes retain their rest placement.

Maps and projection images are explicit owning-module paths under `assets/` or `sources/`. The cook captures their bytes once. Model `process` runs repair, decimation, UV preparation/baking, projection, transfer and skin binding on native input. Authored model `options` owns LODs, collision, raycast acceleration, impostors, texture streaming, animation keys and skin relaxation; there are no sibling settings. Animation rig and library refs are canonical content IDs.

## 2. Building one

1. Start from the look's face budget, normals, bevel and palette. Budget rebuilding reduces generator detail rather than silently dropping parts. Write the counts the style wants, not dense placeholders.
2. Block the silhouette with three to six primitives at the right proportions. Check front and side views before adding detail.
3. Repeat placements rather than copying meshes. Put moving parts under pivots at their hinge or axle, not at their geometric centre.
4. Prefer an extrusion with a hole to CSG when both express the shape: CSG adds faces along each cut.
5. Review a turntable and fix the largest silhouette or material error first (`art-review`).

## 3. Plants

A plant item grows wood, leaves and petals from explicit growth parameters and a seed, with vertex colours and wind data. It is one model at its root. Set `og.Model.sway` and `og.Wind` to move it. Each file grows one plant so wind bends it around one root.

```ts
import {file} from '@origingame/engine/author';
export default file.model({kind:'geometry',desc:{
  options:{impostor:true},
  items:[{name:'birch',input:{kind:'plant',desc:{
    kind:'birch',seed:12,
    leaves:{color:[0.5,0.3,0.02],color2:[0.6,0.14,0.02]}
  }}}]
}});
```

Kinds are growth starting points, not gameplay: oak, birch, willow, pine, palm, bamboo, cactus, shrub, fern, reed, daisy and tulip. Override only the shape that differs; nested leaf/flower/level inputs retain the kind's other values. Seeds give different individuals. A plant without a kind starts as a small broadleaf tree, with leaves and flowers only when specified. Use parts or an import for a shape none of these mechanisms describes.

Growth follows the trunk and up to three branch levels. Evergreen growth shortens boughs toward the leader; forces bend or droop limbs. Leaves may be procedural blades or textured cards/crosses; named leaf materials own translucency, rounded normals and seasonal colours. Two colours vary leaves or petals within one individual. Seasonal metadata is retained on materials; it does not itself advance a game calendar.

An explicit branch list replaces growth. Parent indices name earlier branches; path points and per-ring radii preserve the designed centreline. A guided trunk path can still grow procedural foliage. Bounds constrain centrelines, not foliage triangles, and reject an outside explicit skeleton instead of moving it. Custom leaf and flower geometries accept the same native geometry union as a part.

A crown shades its interior from leaf density and volume. The meshlet and terrain-scatter paths retain canopy light and flutter; an impostor holds this light in its bake. Round canopy normals gather diffuse light while each leaf's own face reflects the sky, avoiding a shiny outline around inner leaves.

Leaves dominate triangle cost: branch children multiply down the levels, then by leaf count. Distant levels thin leaves/petals and enlarge retained ones to keep crown coverage. Small plants retain all leaves. Near-camera stands should first reduce count and increase size, then reduce branch children. The cook refuses more than 20,000 branches or 120,000 leaves and petals.

Far scattered copies can use an impostor, enabled in the declaration's `options`. At 128 pixels per frame, an impostor reads soft when its texels exceed screen pixels. For a 50° view, start at the plant's radius × screen height / 60: about 110 m for a 9 m-radius oak at 720 rows, or 28 m for a 2.3 m-radius shrub. Increasing frame size brings that boundary nearer and grows atlas area quadratically. Below-horizon views need the lower hemisphere, doubling atlas width. Cards sway around the model's foot.

A placed `og.Model` cross-fades mesh LOD/impostor changes over 0.4 s; posed, crowd and scatter mesh LODs switch directly. Wind strength is lean per metre: a 10 m tree needs much less sway than grass. Give trees their own lower sway when they share a wind field.

## 4. Pitfalls

- Colours are linear. Convert an sRGB sample or use the look's palette.
- Extrusion bevels grow the outer size; shrink the outline to compensate.
- Rigid-node keys animate named pivots and parts. Humanoid motion requires a skin, whether imported or explicitly bound in processing.
- Unwrapping textured geometry requires baking its appearance onto the new UVs; processing rejects existing clips, skins and morphs rather than discarding them.
- LODs, raycast acceleration, collision and model loading need no export step. An authored model is native input to the normal model cook.
