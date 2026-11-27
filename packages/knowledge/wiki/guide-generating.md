<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `generating` guide: Generating source data with files

Keep the recipe in the project: use an asset's existing generation mechanism when it has one, and a deterministic TS module when its input needs a designed shape or a composition of mechanisms. Cook writes the normal asset; the game loads it without running its generator again. These are mechanisms, not terrain or gameplay presets.

For emitter/material/composed-effect families, `type particles`, `type effect` and `type vfx` accept one exclusive script `generate` block. Return `EffectSource {text}` containing the target format's body declarations. The same executor, import/input tracking and BLAKE3 cache described here run it; ordinary effect parsers validate and link the result. `effects §5` contains a checked recipe that generates all three and composes them with a TS `file["effect-texture"]({...})` volume under content/. The volume's old OGT author entry is removed; its output is `<module>/<content-relative-stem>` with real sheet, array and layer subresources. Noise/dissolve/flow textures use the shared procgen noise, Worley and curl kernels; smoke, fire and explosions use offline density/heat transport. No source generator needs to write PNG files or encode cooked assets itself. Timelines instead export `file.timeline` directly (`animation §13`); no text generation bridge is accepted.

Cook scripts may import `{ input, procgen }` from `@origingame/engine/author`. Call `input.bytes("height.bin")` or `input.text("layout.csv")` directly with a string literal; paths are relative to the evaluator's input root, currently `assets/` for asset generators, even from a nested helper. Cook captures these files before evaluation and tracks their bytes with the helper imports. Bytes calls return independent `Uint8Array` copies; text requires UTF-8 and preserves BOM/line endings. Named import aliases work, but assigning the input capability to another variable, re-exporting it, or computing a path does not. Missing files, symlinks and paths outside the root fail. This module does not yet replace the asset declarations described below.

For random source data, import `rng` from the same author module and supply a u32 seed: `const stream = rng(42)`. `stream.next()` draws in [0, 1) using the engine's Rust RNG; `stream.fork("cliffs")` makes an independent stream derived from the seed and UTF-16 label, without advancing its parent. Repeated forks with the same label start the same sequence, regardless of the parent's draw count. Streams are local to the cook realm, not runtime handles or simulation state. There is no implicit project seed or global draw counter; `Math.random()` remains forbidden. The RNG implementation participates in the cook cache key.

The author module also exposes schema-derived `file.<kind>(value)` factories and `ref(id)` content identities. A factory returns a descriptor, not a cooked asset or runtime handle; constructing a reference never runs another content module. Hyphenated kinds use brackets, such as `file["texture-array"](...)`. `og api gen/check` generate and drift-check the engine's `types/author-inputs.d.ts` from its Rust input schemas. Contextual scene, prefab and table types require project-specific record binding rather than accepting arbitrary fields. These are author frontend primitives; the asset declarations and generation blocks below still use their existing cook consumers until each format is cut over.

Content discovery keeps the source `factory` separate from its primary reference `kind`. The domain schema's `x-og-output-kind` declares a different primary kind: `model-program` produces `model`, `effect-texture` produces `texture`, `sfx` produces `sound`, `vfx2d` produces `flipbook`, `theme` produces `pack` and `font-extension` produces `font-pack`. Otherwise the primary kind equals the factory name; import entries use their literal schema-defined discriminant. `x-og-ref-kinds` accepts output kinds only, without source-name aliases. Model `program` and theme `extends` instead use `x-og-ref-factories` to require the original program/recipe declaration: an imported model, an ordinary pack or a derived output is not its source. Derived suffixes and their kinds come from actual domain producers; discovery never guesses them. TS field, sound and effect-texture declarations have native cooker adapters. Other domains keep their existing author entry until their complete cutover; a schema or factory being listed does not mean its native adapter has shipped.

Cook transfers and caches source data as canonical binary author IR. Numeric typed arrays keep their element type and only their selected view's bytes; JSON expansion happens only for consumers that request it. Each entry has a 64 MiB encoded-output budget, one million structural nodes and 128 nested containers. Typed array samples do not count as separate structural nodes. Split a larger world into independently cooked tiles or content entries instead of increasing a single entry's budget. Non-finite numbers, sparse arrays, extra typed-array properties, getters, class instances, promises and cyclic objects remain errors.

<a id="guide-generating-1"></a>
### `generating §1` Find the file contract

`og api list --kind file` lists the registered cook types and the other engineering sources. `og api show file:terrain` describes terrain fields; `og api show file:terrain.generate.erosion` isolates a nested block. `og api find erosion --kind file` searches this namespace. Each entry states its field-coverage status: a type being registered does not mean its full field reference has been migrated yet. UI stays TSX; model programs and themes retain their existing JSON sources.

<a id="guide-generating-2"></a>
### `generating §2` Design heights, then process them

The following complete recipe includes its game entry, a terrain asset and TS heightfield generator, and a model with curve primitives and a TS mesh source. The heightfield generator makes a deliberately asymmetric rectangular field in metres. The terrain applies thermal erosion. The game compares sampled cooked heights with the same shared processing mechanism at runtime and inspects the cooked model; quantisation is bounded by the terrain's stored height range. No Python, subprocess or second erosion implementation is involved.

```ts recipe ticks=2
import { og } from "@origingame/engine";
import heights from "../assets/heights.ts";

export default og.game({
  start(world) {
    const base = heights({ params: { offset: 117 }, seed: 9, inputs: {} });
    const preserve = new Float32Array(base.columns * base.rows);
    preserve[3 * base.columns + 8] = 1;
    preserve[3 * base.columns + 6] = 0.5;
    const result = og.procgen.heightmap({ ...base, size: { x: 24, y: 12 }, thermal: { talus: 20, iterations: 3, rate: 0.3 },
      preserve, preserveBorder: 1, channels: [
        { points: [{x:-8,y:120,z:0}, {x:8,y:116,z:0}], width:8 },
        { points: [{x:-10,y:120,z:-4}, {x:0,y:118,z:0}, {x:10,y:116,z:4}], width:2, depth:1, river:true },
      ],
    });
    og.assert.always(result.ok, "runtime processing accepted", {});
    if (!result.ok) return;
    const made = result.value;
    const loaded = og.terrain.load(world, "surface.ogt");
    og.assert.always(loaded.ok, "cooked terrain loads", {});
    if (!loaded.ok) return;
    const spawned = world.spawn({ name: "ground" });
    if (!spawned.ok) throw new Error(spawned.reason);
    const ground = spawned.value;
    world.insert(ground, og.Terrain, { data: loaded.value.data });
    world.insert(ground, og.Collider, { shape: "heightfield", mesh: loaded.value.geometry });
    const tolerance = (made.maxHeight - made.minHeight) / 65535 + 0.001;
    for (const [c, r] of [[0, 0], [6, 2], [12, 6], [3, 5], [8, 3], [6, 3]]) {
      const sample = og.terrain.sample(world, { data: loaded.value.data, x: c! * 2 - 12, z: r! * 2 - 6 });
      og.assert.always(sample.ok && Math.abs(sample.value!.height - made.heights[r! * base.columns + c!]!) <= tolerance, "cooked and runtime heights agree", {});
    }
    og.assert.always(Math.abs(made.heights[0]! - 117) < 0.001 && made.maxHeight > 125, "authored metres are not normalised or clipped to the noise range", {});
    og.assert.always(made.heights[3 * base.columns + 8] === base.heights[3 * base.columns + 8], "full shape constraint preserves the designed peak exactly", {});
    og.assert.always(Math.abs(made.heights[3 * base.columns + 6]! - (base.heights[3 * base.columns + 6]! + 117) / 2) < 0.001, "half constraint blends the authored river bed toward the input", {});
    og.assert.always(Math.abs(made.riverLength - 2 * Math.sqrt(116)) < 0.001, "declared river segments are measured in terrain metres", {});
    const edited = og.terrain.load(world, "edited.ogt");
    if (!edited.ok) throw new Error(edited.reason);
    for (const [x, z, expected] of [[-8,-4,23], [4,0,9], [2,0,7], [10,4,20]]) {
      const sample = og.terrain.sample(world, { data: edited.value.data, x: x!, z: z! });
      og.assert.always(sample.ok && Math.abs(sample.value!.height - expected!) < 0.002, "cook applies authored brushes and model stamps in written order", {});
    }
    const model = og.load.gltf(world, "curves.ogt", {});
    if (!model.ok) throw new Error(model.reason);
    const info = og.load.gltfInfo(world, model.value);
    if (!info.ok) throw new Error(info.reason);
    og.assert.always(info.value.parts.length === 5, "all authored curve and buffer primitives cooked", {});
    for (const [index, triangles] of [8, 32, 72, 80, 1].entries()) {
      const geometry = og.render.geometryInfo(world, info.value.parts[index]!.geometry);
      og.assert.always(geometry.ok && geometry.value!.triangles === triangles, "curve tessellation preserved", {});
    }
    const generated = og.render.geometryInfo(world, info.value.parts[4]!.geometry);
    og.assert.always(generated.ok && generated.value!.min.x === -2 && generated.value!.max.x === 3 && generated.value!.max.y === 4, "authored mesh dimensions preserved", {});
    const scene = og.load.scene(world, "placements.ogt");
    if (!scene.ok) throw new Error(scene.reason);
    const placed = world.instantiate(scene.value, { name: "placements" });
    if (!placed.ok) throw new Error(placed.reason);
    const marker = world.find("marker");
    if (!marker) throw new Error("generated entity was not instantiated");
    const transform = world.get(marker, og.Transform);
    og.assert.always(transform.ok && transform.value!.position.x === 2 && transform.value!.position.z === -3 && transform.value!.scale.x === 0.5, "generated fields compose with authored prefab fields", {});
    const scatter = og.load.scene(world, "scatter.ogt");
    if (!scatter.ok) throw new Error(scatter.reason);
    const scattered = world.instantiate(scatter.value, { name: "scatter" });
    if (!scattered.ok) throw new Error(scattered.reason);
    const point = world.find("point-0");
    if (!point) throw new Error("scatter point missing");
    const at = world.get(point, og.Transform);
    if (!at.ok) throw new Error(at.reason);
    const projected = og.terrain.sample(world, { data: loaded.value.data, x: at.value.position.x, z: at.value.position.z });
    og.assert.always(projected.ok && Math.abs(at.value.position.y - projected.value!.height - 0.2) <= tolerance, "scatter projects onto the same terrain surface", {});
    og.assert.always(Math.abs(at.value.scale.x - 0.15) < 0.00001, "scatter preserves explicit scales", {});
    const operand = og.load.gltf(world, "copy.ogt", {});
    if (!operand.ok) throw new Error(operand.reason);
    const copied = og.load.gltfInfo(world, operand.value);
    if (!copied.ok) throw new Error(copied.reason);
    const bounds = og.render.geometryInfo(world, copied.value.parts[0]!.geometry);
    og.assert.always(bounds.ok && bounds.value!.min.x === -2 && bounds.value!.max.y === 6, "source mesh node retains geometry through a placed operand", {});
    const graph = og.load.animgraph(world, "graph.ogt");
    const again = og.load.animgraph(world, "graph.ogt");
    og.assert.always(graph.ok && again.ok && graph.value === again.value, "cooked sprite graph loads once", {});
    const plant = og.load.gltf(world, "guided.ogt", {});
    if (!plant.ok) throw new Error(plant.reason);
    const foliage = og.load.gltfInfo(world, plant.value);
    if (!foliage.ok) throw new Error(foliage.reason);
    og.assert.always(foliage.value.parts.length === 2, "guided plant cooks wood and custom foliage", {});
    const counts = foliage.value.parts.map(part => og.render.geometryInfo(world, part.geometry));
    og.assert.always(counts.every(v => v.ok) && counts.reduce((n, v) => n + v.value!.triangles, 0) === 17, "two trunk sections and one custom leaf triangle survive cook", {});
    const branches = [
      { parent: -1, path: [{x:0,y:0,z:0},{x:0,y:2,z:0},{x:0,y:4,z:0}], radii: new Float32Array([0.2,0.15,0.05]), segments: 6 },
      { parent: 0, path: [{x:0,y:2,z:0},{x:-2,y:3,z:0},{x:-3,y:3,z:1}], radii: new Float32Array([0.1,0.06,0.02]), segments: 5 },
      { parent: 0, path: [{x:0,y:1,z:0},{x:2,y:2,z:-1}], radii: new Float32Array([0.12,0.03]), segments: 4 },
    ];
    const skeleton = og.procgen.plant({branches});
    if (!skeleton.ok) throw new Error(skeleton.reason);
    const skeletonModel = og.load.gltf(world, "skeleton.ogt", {});
    if (!skeletonModel.ok) throw new Error(skeletonModel.reason);
    const skeletonInfo = og.load.gltfInfo(world, skeletonModel.value);
    if (!skeletonInfo.ok) throw new Error(skeletonInfo.reason);
    const skeletonGeometry = og.render.geometryInfo(world, skeletonInfo.value.parts[0]!.geometry);
    og.assert.always(skeleton.value.branches === 3 && skeleton.value.wood.indices.length === 156 && skeletonGeometry.ok && skeletonGeometry.value.triangles === 52, "cook and runtime preserve the explicit asymmetric branch topology", {});
    const terminal = og.procgen.plant({branches, leaves:{count:0,end:1,sizeVariance:0}});
    og.assert.always(terminal.ok && terminal.value.leafCount === 2, "only the two terminal skeleton limbs receive end leaves", {});
    const unattached = og.procgen.plant({branches: [branches[0]!, {...branches[1]!, path: [{x:1,y:2,z:0},{x:-2,y:3,z:0},{x:-3,y:3,z:1}]}]});
    og.assert.always(!unattached.ok, "disconnected authored branches are refused", {});
    const pruned = og.procgen.plant({levels:0, forceStrength:0,
      levelsSpec:[{length:4,radius:0.1,sections:4,segments:6,taper:0,gnarliness:0}],
      bounds:{min:{x:-1,y:-0.1,z:-1},max:{x:1,y:2.5,z:1}},
    });
    if (!pruned.ok) throw new Error(pruned.reason);
    let top = -Infinity;
    for (let i=1; i<pruned.value.wood.positions.length; i+=3) top = Math.max(top, pruned.value.wood.positions[i]!);
    og.assert.always(Math.abs(top - 2) < 0.00001, "growth stops before the first outside centreline ring", {});
    const processed = og.load.gltf(world, "processed.ogt", {});
    if (!processed.ok) throw new Error(processed.reason);
    const prepared = og.load.gltfInfo(world, processed.value);
    if (!prepared.ok) throw new Error(prepared.reason);
    const reduced = og.render.geometryInfo(world, prepared.value.parts[0]!.geometry);
    og.assert.always(reduced.ok && reduced.value!.triangles > 0 && reduced.value!.triangles < 168, "authored repair, reduction and atlas baking produce a loadable mesh", {});
    const projectedModel = og.load.gltf(world, "projected.ogt", {});
    if (!projectedModel.ok) throw new Error(projectedModel.reason);
    const projectionInfo = og.load.gltfInfo(world, projectedModel.value);
    og.assert.always(projectionInfo.ok && projectionInfo.value.parts.length === 1, "registered image projection cooks a normal model", {});
    const variant = og.load.gltf(world, "variant.ogt", {});
    if (!variant.ok) throw new Error(variant.reason);
    const variantInfo = og.load.gltfInfo(world, variant.value);
    if (!variantInfo.ok) throw new Error(variantInfo.reason);
    const variantShape = og.render.geometryInfo(world, variantInfo.value.parts[0]!.geometry);
    og.assert.always(variantShape.ok && variantShape.value!.max.x === 1.5, "named-part transfer retains the variant geometry", {});
    og.assert.reachable("generated terrain checked", {});
  },
});
```

```ogt recipe file=assets/processed.ogt
type model
version 1
---
conform off
material "surface" {
  map "surface.png"
  normalMap "surface-normal.png"
  roughness 0.8
}
part "surface" {
  material "surface"
  sphereGeometry {
    widthSegments 12
    heightSegments 8
  }
}
process {
  repair {
    weld 0
  }
  decimate {
    ratio 0.6
  }
  unwrap {
    resolution 64
  }
  bake {
    size 64
    padding 2
  }
  rig {
    joint "root" "" 0 -1 0
    joint "tip" "root" 0 1 0
  }
}
```

The model's `process` block runs repair, decimation, UV preparation/material baking and skin binding in that order, through the existing mesh kernels. It accepts static meshes only; it refuses existing animation, skins and morphs rather than discarding them. An explicit skeleton uses model-local metres, Y up. Unwrapping a textured input requires baking so its authored appearance survives the UV replacement. Processed maps are embedded and their source bytes remain cook dependencies.

`process.project` projects author-supplied, camera-registered square images into the prepared UVs with the existing projection kernel. It is a deterministic processing step, not image generation: there is no implicit silhouette fitting, view selection or network request. Camera positions use local metres, Y up; `ortho` is half the image's height in metres. All image bytes join the action's dependency key. Unseen texels use the existing nearest-painted-surface fill. Other material channels stay unchanged.

```ogt recipe file=assets/projected.ogt
type model
version 1
---
conform off
part "panel" {
  planeGeometry {
    width 2
    height 2
  }
}
process {
  project {
    size 32
    padding 2
    view "registered.svg" {
      position 0 0 4
      target 0 0 0
      ortho 1
    }
  }
}
```

```text recipe file=assets/registered.svg
<svg xmlns="http://www.w3.org/2000/svg" width="32" height="32"><rect width="32" height="32" fill="#e34a2c"/><rect x="24" width="8" height="32" fill="#2c69df"/></svg>
```

`process.transfer` carries an imported base model's painted maps to its variant using the same part-local metric mechanism as the model bake command. Stable part names identify corresponding surfaces. Each part anchors at its own minimum corner, in metres, without fitting or rescaling: an enlarged region extends the nearest same-facing base region. The source's normal importer, processing and external image dependencies run first; cycles fail. Both models need prepared UVs. Source painted maps must be square and share one size; destination `size` is independent. Only base colour changes; other material channels stay authored. Projection runs before transfer when both are declared.

```ogt recipe file=assets/variant.ogt
type model
version 1
---
conform off
part "panel" {
  planeGeometry {
    width 3
    height 2
  }
}
process {
  transfer {
    source "projected.ogt"
    size 32
    padding 2
  }
}
```

A material library's explicit `generate` block invokes the existing periodic/PBR map preparation during cook. Destinations must not collide with authored files or other outputs. Generated PNG inputs exist in the cook graph, not in `assets/`; model, processing and fracture consumers resolve the same bytes. The image and every processing parameter form the action key. `og materials make` remains the explicit authoring command when source files are wanted.

```ogt recipe file=assets/materials.ogt
type materials
version 1
---
material "surface" {
  label surface
  generate {
    source "surface.svg"
  }
  tile 2 1
  relief 0.02
  map "surface.png"
  normalMap "surface-normal.png"
}
```

```text recipe file=assets/surface.svg
<svg xmlns="http://www.w3.org/2000/svg" width="32" height="16"><rect width="32" height="16" fill="#bf7944"/><path fill="#483025" d="M3 0h5v16H3zM19 0h3v16h-3z"/></svg>
```

```ogt recipe file=assets/surface.ogt
type terrain
version 1
---
size 24 12
tile 16
generate {
  script "heights.ts"
  seed 9
  params {
    offset 117
  }
  thermal {
    talus 20
    iterations 3
    rate 0.3
  }
  preserve "base/protect"
  preserveBorder 1
  channel {
    point -8 120 0
    point 8 116 0
    width 8
  }
  channel {
    point -10 120 -4
    point 0 118 0
    point 10 116 4
    width 2
    depth 1
    river true
  }
}
layer {
  color 0.32 0.43 0.38
  roughness 0.9
}
```

```ts recipe file=assets/heights.ts
import { procgen, type GeneratorContext, type HeightfieldSource } from "@origingame/engine/generate";

export default function heights(ctx: GeneratorContext<{ offset: number }>) {
  const columns = 13, rows = 7;
  const heights = new Float32Array(columns * rows);
  for (let r = 0; r < rows; r++) {
    for (let c = 0; c < columns; c++) {
      const detail = 0.05 * c * procgen.noise({ seed: ctx.seed }, c / 7, r / 3, 0);
      heights[r * columns + c] = ctx.params.offset + 0.7 * c + 0.2 * r + (c === 8 && r === 3 ? 9 : 0) + detail;
    }
  }
  return { columns, rows, heights } satisfies HeightfieldSource;
}
```

```ogt recipe file=assets/edited.ogt
type terrain
version 1
---
size 24 12
tile 16
generate {
  script "flat.ts"
}
edit {
  mode raise
  center -8 0 -4
  radius 1
  strength 3
}
stamp {
  modelGeometry {
    source "stamp.ogt"
  }
  position 4 8 0
}
edit {
  mode raise
  center 4 0 0
  radius 1
  strength 2
}
```

```ts recipe file=assets/flat.ts
export default () => ({ columns:13, rows:7, heights: new Float32Array(91).fill(20) });
```

```ogt recipe file=assets/stamp.ogt
type model
version 1
---
conform off
part "underside" {
  boxGeometry {
    width 4
    height 2
    depth 4
  }
}
```

```ts recipe file=content/protect.ts
import { file } from "@origingame/engine/author";
export default file.field((() => {
  const values = new Float32Array(13 * 7);
  values[3 * 13 + 8] = 1;
  values[3 * 13 + 6] = 0.5;
  return { dimensions: [13,7,1] as const, channels: 1, values };
})());
```

```ogt recipe file=assets/guided.ogt
type model
version 1
---
conform off
plant "guided" {
  levels 0
  path -2 0 1 -1 2 1 1 3 -1
  trunk {
    radius 0.2
    segments 4
  }
  leaves {
    count 1
    start 0.5
    end 0.5
    sizeVariance 0
  }
  leafGeometry {
    bufferGeometry {
      generate {
        script "leaf.ts"
      }
    }
  }
}
```

An explicit `branch` list replaces procedural growth altogether. The first limb is the root; children name an earlier limb and start on its centreline. Per-ring radii retain the authored silhouette, and terminal limbs receive the same leaves, flowers and wind data. `bounds` limits centreline growth, not the triangles of wood or foliage; it rejects an outside explicit skeleton rather than silently changing that design.

```ogt recipe file=assets/skeleton.ogt
type model
version 1
---
conform off
plant "skeleton" {
  bounds -4 -1 -2 3 5 2
  branch {
    parent -1
    path 0 0 0 0 2 0 0 4 0
    radii 0.2 0.15 0.05
    segments 6
  }
  branch {
    parent 0
    path 0 2 0 -2 3 0 -3 3 1
    radii 0.1 0.06 0.02
    segments 5
  }
  branch {
    parent 0
    path 0 1 0 2 2 -1
    radii 0.12 0.03
    segments 4
  }
}
```

```ts recipe file=assets/leaf.ts
import type { MeshSource } from "@origingame/engine/generate";
export default () => ({
  positions: new Float32Array([0,0,0,1,0,0,0,1,0]),
  uvs: new Float32Array([0,0,1,0,0,1]),
  colors: new Float32Array([1,1,1,0.25,1,1,1,0.5,1,1,1,0.75]),
}) satisfies MeshSource;
```

```ogt recipe file=assets/curves.ogt
type model
version 1
---
conform off
part "disc" {
  circleGeometry {
    radius 2
    segments 8
    thetaLength 3.141592653589793
  }
}
part "ring" {
  ringGeometry {
    innerRadius 0.7
    outerRadius 2.3
    thetaSegments 8
    phiSegments 2
  }
}
part "polyline" {
  tubeGeometry {
    points 0 0 0 1 2 0 3 2 1
    radius 0.2
    tubularSegments 9
    radialSegments 4
  }
}
part "curve" {
  tubeGeometry {
    path {
      kind catmullRom
      point 0 0 0
      point 1 1 1
      point 3 2 0
    }
    radius 0.3
    tubularSegments 8
    radialSegments 5
  }
}
part "buffer" {
  bufferGeometry {
    generate {
      script "mesh.ts"
    }
  }
}
```

```ts recipe file=assets/mesh.ts
import type { MeshSource } from "@origingame/engine/generate";

export default function mesh() {
  return {
    positions: new Float32Array([-2, 0, 0, 3, 0, 0, 1, 4, 0]),
    colors: new Float32Array([1, 0, 0, 0, 1, 0, 0, 0, 1]),
    uvs: new Float32Array([0, 1, 1, 1, 0.6, 0]),
    tangents: new Float32Array([1, 0, 0, -1, 1, 0, 0, -1, 1, 0, 0, -1]),
    indices: new Uint32Array([0, 1, 2]),
  } satisfies MeshSource;
}
```

```ogt recipe file=assets/placements.ogt
type scene
version 1
---
generate {
  script "placements.ts"
}
```

```ts recipe file=assets/placements.ts
import type { SceneSource } from "@origingame/engine/generate";

export default function placements() {
  return { entities: [{
    name: "marker", prefab: "marker.ogt",
    components: { Transform: { position: [2, 1, -3] } },
  }] } satisfies SceneSource;
}
```

```ogt recipe file=assets/marker.ogt
type prefab
version 1
---
entity "marker" {
  model "curves.ogt"
  Transform {
    scale 0.5 0.5 0.5
  }
}
```

```ogt recipe file=assets/scatter.ogt
type scatter
version 1
---
model "curves.ogt"
project "surface.ogt"
align true
generate {
  script "scatter.ts"
  seed 17
}
```

```ogt recipe file=assets/copy.ogt
type model
version 1
---
conform off
part "copy" {
  mergeGeometries {
    part {
      modelGeometry {
        source "curves.ogt"
        node "buffer"
      }
      position 0 2 0
    }
  }
}
```

```ogt recipe file=assets/graph.ogt
type animgraph
version 1
---
sprite true
clip still "still"
layer base {
  state idle still
  entry idle
}
```

```ts recipe file=assets/scatter.ts
import { procgen, type GeneratorContext, type ScatterSource } from "@origingame/engine/generate";

export default function scatter(ctx: GeneratorContext) {
  const points = procgen.poissonDisk({ width: 20, height: 8, radius: 3, maxPoints: 12, seed: ctx.seed });
  if (!points.ok) throw new Error(points.reason);
  return {
    positions: Float32Array.from(points.value.flatMap(p => [p.x - 10, 0.2, p.y - 4])),
    scales: new Float32Array(points.value.length * 3).fill(0.15),
  } satisfies ScatterSource;
}
```

The TS default export synchronously returns `{columns, rows, heights}`. Samples are row-major from -z to +z and -x to +x; dimensions are 2..4097, every height is finite, and `size` defines positive x/z extents. A supplied field is never implicitly resampled or normalised. It cannot also request a noise source (`samples`, `noise`, `mask`) or intensity-to-height conversion (`height`). Relative static imports must name their extensions and stay inside the asset root.

For an existing height image, write `heightmap "base.png"` (or EXR) beside `generate`. PNG intensity maps through `height low high`; EXR defaults to metres (`height 0 1`). The same processing runs after this conversion. Without a supplied field, `generate` starts with noise and maps it into `height` instead. Processing order is hydraulic erosion → thermal erosion → terraces → shore → automatic drainage/rivers → biome control. On supplied fields terraces anchor at zero and erosion may leave the original extrema. Author-directed valley lines, river networks and protection masks are separate inputs, not implied by automatic drainage.

<a id="guide-generating-3"></a>
### `generating §3` Script, inputs and caching

`og script heights.ts --root assets --params '{"offset":117}' --seed 9 --out og-out/heights.json --json` executes the very same generator as cook. `og mcp` exposes the same `script` tool. `--out` is an explicit JSON output, not permission for the TS module to write files. Run from the recipe project directory; parent directories are created and the complete output is published by rename.

Use repeated `input "name" "relative/path"` entries inside `generate` (or `--input name=path` with script) for source bytes. Only those bytes are exposed at `ctx.inputs[name]`; the script cannot open paths. Parameters, seed, source, transitive imports, declared input contents and relevant engine/toolchain identity determine the shared cook cache key. Moving an unchanged project does not change its key. A source/input change invalidates it even when its timestamp was preserved.

The realm has no filesystem, process, environment, network, system clock or timers. `Math.random` is refused: use an explicitly seeded algorithm and `ctx.seed`. Dynamic code compilation is unavailable; mathematical transcendentals use software libm. Imports are closed before execution. Dynamic import and `await` are rejected at their source position, including unused branches. Type-only imports are erased before collecting executable dependencies; the project type checker resolves their declarations. Heap, stack, instruction, module and input budgets stop runaway work.

Outputs contain finite numbers, strings, booleans, null, plain objects and dense arrays; numeric typed arrays become arrays. Optional fields must be absent, not `undefined`. Functions, symbols, BigInt, promises, class instances (including Map/Set), sparse arrays, extra array properties, hidden/accessor object properties and cycles are rejected with the file and value path. Validation does not execute getters or `toJSON`. Shared plain data is allowed when it is not cyclic. Object keys are canonicalised; array order is preserved. Invalid source data fails cook before the target asset is published. Runtime handles and world snapshots are not source data.

<a id="guide-generating-4"></a>
### `generating §4` Compose the shared mechanisms

`@origingame/engine/generate` exports `procgen` with exactly the signatures documented for `og.procgen`: noise, noiseGrad, curlNoise, worley, noiseGrid, poissonDisk, bsp, wfc, heightmap, plant and fracture. Cook calls the same Rust implementation and IDL parameter reader as the game hosts, without creating a world. An imported generator can also be called by a game: the module then forwards to its normal runtime bindings, as the recipe does above.

Typed buffers remain typed inside the generator, including nested mesh attributes and WFC/BSP grids; a `subarray` denotes only that slice. At the final JSON output boundary they become numeric arrays. Result-returning mechanisms keep `{ok, value}` / `{ok: false, reason}`: check `ok` before composing them. These pure mechanisms do not load assets or return runtime handles. Existing files remain the normal input for plants, sound synthesis and prefracture; calling a mechanism in TS does not bypass a target asset's source validation.

A model's `bufferGeometry.generate` returns `MeshSource`: flat vertex attribute arrays with the same names and validation as `og.render.bufferGeometry`, not a runtime geometry handle. Materials, pivots and placements stay in the surrounding model file. The first recipe cooks an asymmetric coloured triangle with explicit tangent frames. Missing normals are computed by the shared implementation; explicit UVs, colours and tangent frames survive normal model import. `bufferGeometry` also accepts authored arrays directly, or participates in nested merges and CSG. Placed merges transform frames with the geometry; CSG creates new topology and regenerates frames when needed.

The guide gate writes all attached files and explicitly cooks the project before running the assertions twice. An attached fence's `file=` is project-relative and cannot escape the recipe directory; it belongs to the preceding `ts recipe` main entry. The guide contains no per-recipe shell harness.

<a id="guide-generating-5"></a>
### `generating §5` Cook a generated tile grid

A `tilemap.generate` script returns `TilemapSource`: positive `width` and `height`, and named layers of row-major tile ids (`-1` empty, not packed id-plus-one cells). Each name must identify one authored layer without `row` lines. The file retains tilesets, orientation, objects, collision and drawing properties; an optional `size` must match the result. Other layers can keep their handwritten rows. A terrain layer takes 0/1 occupancy instead. Invalid dimensions, ids or layer names fail cook.

This recipe composes WFC and BSP into two layers. The script explicitly maps mechanism output to the file's tile ids. WFC's sockets state adjacency, not a built-in wall or road interpretation; when enabling rotations, map the returned quarter-turns to authored tile variants too. At runtime only the cooked map is loaded.

```ts recipe ticks=2
import { defineSystem, og } from "@origingame/engine";

const Check = defineSystem("Check", { reads: [og.Tilemap], writes: [] }, (_q, _time, world) => {
  const entity = world.find("grid");
  if (!entity || og.tilemap.getTile(world, entity, "sockets", 0, 0) < 0) return;
  let marked = 0, empty = 0;
  for (let y = 0; y < 7; y++) {
    for (let x = 0; x < 12; x++) {
      const tile = og.tilemap.getTile(world, entity, "sockets", x, y);
      og.assert.always(tile === (x % 2 === 0 ? 2 : 0), "fixed sockets cooked to authored tile ids", {});
      const region = og.tilemap.getTile(world, entity, "regions", x, y);
      og.assert.always(region === -1 || region === 1, "explicit BSP tile mapping preserved", {});
      if (region === 1) marked++; else empty++;
    }
  }
  og.assert.always(marked > 0 && empty > 0, "BSP produces both marked and empty cells", {});
  og.assert.reachable("generated grid adopted", {});
});

export default og.game({
  systems: [Check],
  start(world) {
    const loaded = og.load.tilemap(world, "grid.ogt");
    if (!loaded.ok) throw new Error(loaded.reason);
    og.assert.always(loaded.value.width === 12 && loaded.value.height === 7, "generated dimensions preserved", {});
    og.assert.always(loaded.value.layers[1]!.parallax.x === 0.5, "authored layer metadata preserved", {});
    const entity = world.spawn({ name: "grid" });
    if (!entity.ok) throw new Error(entity.reason);
    world.insert(entity.value, og.Tilemap, { map: loaded.value.map });
    const sound = og.load.sound(world, "base/signal");
    const sampled = og.load.sound(world, "base/sampled");
    og.assert.always(sampled.ok, "sample-based synthesis cooks as a normal sound", {});
    const source = { samples: [0.25], rate: 8000, reference: 440, loop: true };
    const layer = { sample: source, freq: 880, attack: 0, sustain: 0.01, decay: 0, gain: 1 };
    const looped = og.audio.synth({ seed: 0, layers: [layer] });
    const oneShot = og.audio.synth({ seed: 0, layers: [{ ...layer, sample: { ...source, loop: false } }] });
    if (!looped.ok || !oneShot.ok) throw new Error("sample synthesis failed");
    const sampleExpected = 0.25 * (1 - 2 * Math.PI * 15 / 48000) ** 7;
    og.assert.always(Math.abs(looped.value.samples[7]! - sampleExpected) < 0.000001 && oneShot.value.samples[7]! < 0, "PCM loops wrap while non-looping sources end at the tuned sample rate", {});
    if (!sound.ok) throw new Error(sound.reason);
    const format = og.load.soundInfo(world, sound.value);
    og.assert.always(format.ok && format.value!.rate === 22050 && format.value!.channels === 2 && format.value!.frames === 300, "generated PCM is a normal sound asset", {});
    const field = og.load.field(world, "base/samples");
    if (!field.ok) throw new Error(field.reason);
    const expected = og.procgen.noiseGrid({ width: 5, height: 3, x: -2, y: 7, step: 0.25, noise: { seed: 17 } });
    if (!expected.ok) throw new Error(expected.reason);
    og.assert.always(field.value.values instanceof Float32Array && field.value.dimensions.join() === "5,3,1" && field.value.channels === 1, "rectangular field shape and typed values load", {});
    og.assert.always(field.value.origin[0] === -2 && field.value.origin[1] === 7 && field.value.spacing[0] === 0.25, "field coordinates preserved", {});
    og.assert.always(field.value.values.every((value, i) => value === expected.value[i]), "cooked field uses the runtime noise mechanism without quantization", {});
  },
});
```

```ogt recipe file=assets/grid.ogt
type tilemap
version 1
---
tile 8 8
tileset "tiles.ogt"
generate {
  script "grid.ts"
  seed 9
  params {
    width 12
    height 7
  }
}
layer "sockets" {
  lit false
}
layer "regions" {
  parallax 0.5 0.8
  lit false
}
```

```ts recipe file=assets/grid.ts
import { procgen, type GeneratorContext, type TilemapSource } from "@origingame/engine/generate";

export default function grid(ctx: GeneratorContext<{ width: number; height: number }>) {
  const { width, height } = ctx.params;
  const wfc = procgen.wfc({ width, height, seed: ctx.seed, tiles: [
    { weight: 1, up: 0, right: 1, down: 0, left: 2, rotations: false },
    { weight: 1, up: 0, right: 2, down: 0, left: 1, rotations: false },
  ], allowed: Array.from({ length: width * height }, (_, i) => [i % width % 2]) });
  const mask = Uint8Array.from({ length: width * height }, (_, i) => i % width === 6 && Math.floor(i / width) < 4 ? 0 : 1);
  const bsp = procgen.bsp({ width, height, seed: ctx.seed, minRoom: 2, maxRoom: 4, depth: 2, padding: 1, corridor: 1, mask,
    rooms: [{ x: 1, y: 1, width: 2, height: 2 }, { x: 8, y: 2, width: 2, height: 2 }] });
  if (!wfc.ok) throw new Error(wfc.reason);
  if (!bsp.ok) throw new Error(bsp.reason);
  return { width, height, layers: [
    { name: "sockets", tiles: Int32Array.from(wfc.value.tiles, tile => tile === 0 ? 2 : 0) },
    { name: "regions", tiles: Int32Array.from(bsp.value.cells, cell => cell === 0 ? -1 : 1) },
  ] } satisfies TilemapSource;
}
```

```ogt recipe file=assets/tiles.ogt
type tileset
version 1
---
texture "tiles.svg"
grid 8 8 3 1
```

```text recipe file=assets/tiles.svg
<svg xmlns="http://www.w3.org/2000/svg" width="24" height="8"><path fill="#285c9b" d="M0 0h8v8H0z"/><path fill="#dbb755" d="M8 0h8v8H8z"/><path fill="#a8493f" d="M16 0h8v8h-8z"/></svg>
```

<a id="guide-generating-6"></a>
### `generating §6` Cook custom PCM

Use a TS `file.sound({...})` declaration under `content/` for custom samples. Specify `rate` in 8000..192000 Hz, `channels` 1 or 2 and interleaved float `samples`, with at least one whole frame. There is no OGT `sound` author entry or generator text wrapper. The native sound importer shares float clamping and quantization with WAV import, then the normal encoder and automatic streaming; no script writes or encodes a WAV. The map recipe above loads this short, asymmetric stereo signal at its canonical ID and checks its format. Existing oscillator synthesis stays in the separate `sfx` domain.

```ts recipe file=content/signal.ts
import { file } from "@origingame/engine/author";

const samples = Float32Array.from({ length: 600 }, (_, i) => [-2, 0.25, 0.5, -0.75, 2, -0.5][i % 6]!);
export default file.sound({ rate: 22050, channels: 2, samples });
```

SFX layers can also use a recording instead of an oscillator. A sample is either `{kind:"file", source:"assets/recording.wav"}` with an explicit canonical module-relative path, or `{kind:"pcm", samples, rate}` with mono samples. Stereo files average to mono. `reference` gives the layer frequency for original-speed playback, and `looping` wraps only the source, not the explicit envelope. Slide, vibrato, filters and drive use the same synth as oscillators; FM is oscillator-only. The file's bytes participate in the cook cache key even when the declaration did not change.

```ts recipe file=content/sampled.ts
import { file } from "@origingame/engine/author";

export default file.sfx({
  layers: [{
    sample: { kind: "pcm", samples: new Float32Array([0.25, -0.5, 0.75]),
      rate: 8000, reference: 440, looping: true },
    freq: 880, attack: 0.002, sustain: 0.01, decay: 0.004, gain: 0.5,
  }],
});
```

<a id="guide-generating-7"></a>
### `generating §7` Cook sampled fields

Export `file.field({...})` from a TS module under `content/`. The schema and native domain validator check the lattice before caching it. Values are interleaved channels with x fastest, then y, then z; the exact dimensions product times channels is required, at most 16,777,216 scalars. No normalization, resampling or image encoding occurs. Import helpers or sample shared procgen mechanisms directly in TS; there is no Field OGT input or `generate` wrapper. Load the canonical module/content ID with `og.load.field`.

This attachment to the map recipe samples the shared noiseGrid mechanism. A module can similarly sample noiseGrad, Worley or curlNoise at explicit coordinates, choosing its own scalar/vector channels.

```ts recipe file=content/samples.ts
import { file, procgen } from "@origingame/engine/author";

export default file.field((() => {
  const values = procgen.noiseGrid({ width: 5, height: 3, x: -2, y: 7, step: 0.25, noise: { seed: 17 } });
  if (!values.ok) throw new Error(values.reason);
  return {
    dimensions: [5, 3, 1] as const, channels: 1,
    origin: [-2, 7, 0] as const, spacing: [0.25, 0.25, 1] as const,
    values: values.value,
  };
})());
```

A particles file's `field "base/vectors" <strength> <size>` accepts a checked field by its canonical ID: a cubic 3-channel lattice, 1..16 samples a side. Terrain's `generate.preserve` likewise names the canonical ID of a one-channel field matching its height grid; navigation baking and projected scatter share that same checked value. Cook embeds particle vectors without RGB remapping or quantization. Size explicitly maps one tiling source period to emitter-local units; origin/spacing describe source sampling, not the emitter's transform. Changes to declared input bytes or transitive TS imports invalidate the field and any affected consumers. These consumer domains retain their current author syntax until their own cutover, but never read Field OGT or re-evaluate field TS.

`type scatter` uses either repeated `point x y z` lines or a `generate` block returning `ScatterSource`: flat xyz positions, optional xyzw quaternions and xyz scales, one per point (up to 1,000,000). It places exactly one authored `model` or `prefab`; no model or distribution is selected for the author. The first recipe composes the shared Poisson mechanism and explicitly maps its rectangle into terrain-local metres. The normal scene pipeline owns model instancing and prefab expansion. `project "terrain.ogt"` adds the terrain's height to each y; `align true` rotates up to its normal before the authored quaternion. Projection rejects points outside the terrain or in holes instead of silently moving or dropping them. Scene and terrain must use the same local coordinates. Load the result with `og.load.scene` or reference it as a sub-scene.

<a id="guide-generating-8"></a>
### `generating §8` Cook explicit scene declarations

A `scene.generate` module returns `SceneSource`: an entity tree and optional resource fields. Every entity has a name; optional `model`, `scene` and `prefab` are asset paths, `static` marks a model for baking, `components` maps reflected names to field objects, and `children` nests entities. Fields are numbers, booleans, strings or flat arrays, not runtime handles. Generated entities append after authored entities. A resource cannot be both authored and generated. Normal prefab merging, component reflection, asset dependencies and sub-scene cycle checks run on the combined declarations. Scene generation does not create a world.

The first recipe includes these declarations. The generator sets a placement; the authored prefab supplies its model and scale. The game only loads the cooked scene.

<a id="guide-generating-9"></a>
### `generating §9` Reuse model geometry and animation graphs

`modelGeometry` supplies an imported model's static rest geometry to an ordinary part, merge or CSG operand. Name a `source` relative to assets/ and optionally a mesh `node`; without a node it merges the visible default scene. Normal source import and generators run first. Cycles fail cook. Full affine placement, UVs, vertex colours and tangent frames survive merges; topology-changing CSG builds new frames. The outer part owns the material. Skinning and animations are not copied into a static operand.

An animation graph is also an asset: use its existing graph language, then load it with `og.load.animgraph`. Add `bind "graph clip name" "model asset" "model clip name"` for each skeletal clip reference; cook checks those names against the model's own, library and mirrored clips. For a flipbook/2D rig use `sprite true` instead of bindings; clip names are then looked up on the target at playback. The first recipe checks the cooked 2D graph and reused model operand. Graph parameters and playback remain runtime state.

<a id="guide-generating-10"></a>
### `generating §10` Author model-local animation keys

An import sidecar's `animation.clip` appends named clips to the imported model, including a static model. Each channel names a unique model node and `translation`, `rotation`, `scale`, `weights` or `visibility`. Times are nonnegative seconds and strictly increase. Translation is metres, rotation is a nonzero xyzw quaternion, scale is dimensionless, and morph lanes must match the target mesh. Visibility uses step keys of 0 or 1. Cubic-spline keys contain in-tangent, value and out-tangent, with tangents per second. The same glTF resampler, compression, events, libraries and mirrored-clip pipeline cooks authored keys; this does not record GPU neural motion or bake runtime IK.

This recipe creates a static model, adds a clip using only its sidecar, then plays and reads the completed pose. A fixed root pivot keeps the panel's authored local travel separate from root-motion extraction. The game does not reconstruct the keys.

```ts recipe ticks=35
import { og, defineSystem } from "@origingame/engine";

const Verify = defineSystem("Verify", { reads: [og.Skeleton] }, (_query, time, world) => {
  if (time.tick !== 33) return;
  const entity = world.find("animated");
  if (!entity) throw new Error("animated entity missing");
  const animation = og.load.animation(world, "animated.ogt");
  if (!animation.ok) throw new Error(animation.reason);
  const pose = og.anim.getNodeMatrix(world, entity, animation.value.nodes.indexOf("panel"));
  if (!pose.ok) throw new Error(pose.reason);
  const m = pose.value.elements;
  og.assert.always(Math.abs(m[12]! - 2) < 0.001 && Math.abs(m[13]! - 1) < 0.001 && Math.abs(m[0]!) < 0.001 && Math.abs(m[1]! - 1) < 0.001, "authored clip played to its asymmetric final pose", {});
  og.assert.reachable("authored animation checked", {});
});

export default og.game({ systems: [Verify], start(world) {
  const model = og.load.gltf(world, "animated.ogt", {});
  const animation = og.load.animation(world, "animated.ogt");
  const entity = world.spawn({ name: "animated" });
  if (!model.ok || !animation.ok || !entity.ok) throw new Error("authored animation failed to load");
  world.insert(entity.value, og.Model, { asset: model.value });
  world.insert(entity.value, og.Skeleton, { asset: animation.value.skeleton });
  world.insert(entity.value, og.AnimationPlayer, { clip: og.anim.clip(animation.value, "designed"), loop: false, fade: 0 });
} });
```

```ogt recipe file=assets/animated.ogt
type model
version 1
---
conform off
pivot "root" {
}
part "panel" {
  pivot "root"
  boxGeometry {
    width 1
    height 2
    depth 0.5
  }
}
```

```ogt recipe file=assets/animated.ogt.ogt
type import
version 1
---
animation {
  rate 30
  clip "designed" {
    channel "panel" translation {
      key 0 0 0 0
      key 0.5 2 1 0
    }
    channel "panel" rotation {
      key 0 0 0 0 1
      key 0.5 0 0 0.70710678 0.70710678
    }
  }
  event "designed" "end" 0.5
}
```
