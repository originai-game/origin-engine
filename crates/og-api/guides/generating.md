# Generating source data with files

Keep the recipe in the project: use an asset's existing generation mechanism when it has one, and a deterministic TS module when its input needs a designed shape or a composition of mechanisms. Cook writes the normal asset; the game loads it without running its generator again. These are mechanisms, not terrain or gameplay presets.

## 1. Find the file contract

`og api list --kind file` lists the registered cook types and the other engineering sources. `og api show file:terrain` describes terrain fields; `og api show file:terrain.generate.erosion` isolates a nested block. `og api find erosion --kind file` searches this namespace. Each entry states its field-coverage status: a type being registered does not mean its full field reference has been migrated yet. UI stays TSX; model programs and themes retain their existing JSON sources.

## 2. Design heights, then process them

The following complete recipe has three files: its game entry, a terrain asset and a TS heightfield generator. The generator makes a deliberately asymmetric rectangular field in metres. The terrain applies thermal erosion. The game compares sampled cooked heights with the same shared processing mechanism at runtime; quantisation is bounded by the terrain's stored height range. No Python, subprocess or second erosion implementation is involved.

```ts recipe ticks=2
import { og } from "@origingame/engine";
import heights from "../assets/heights.ts";

export default og.game({
  start(world) {
    const base = heights({ params: { offset: 117 }, seed: 9, inputs: {} });
    const result = og.procgen.heightmap({ ...base, size: { x: 24, y: 12 }, thermal: { talus: 20, iterations: 3, rate: 0.3 } });
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
    for (const [c, r] of [[0, 0], [6, 2], [12, 6], [3, 5]]) {
      const sample = og.terrain.sample(world, { data: loaded.value.data, x: c! * 2 - 12, z: r! * 2 - 6 });
      og.assert.always(sample.ok && Math.abs(sample.value!.height - made.heights[r! * base.columns + c!]!) <= tolerance, "cooked and runtime heights agree", {});
    }
    og.assert.always(Math.abs(made.heights[0]! - 117) < 0.001 && made.maxHeight > 125, "authored metres are not normalised or clipped to the noise range", {});
    og.assert.reachable("generated terrain checked", {});
  },
});
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

The TS default export synchronously returns `{columns, rows, heights}`. Samples are row-major from -z to +z and -x to +x; dimensions are 2..4097, every height is finite, and `size` defines positive x/z extents. A supplied field is never implicitly resampled or normalised. It cannot also request a noise source (`samples`, `noise`, `mask`) or intensity-to-height conversion (`height`). Relative static imports must name their extensions and stay inside the asset root.

For an existing height image, write `heightmap "base.png"` (or EXR) beside `generate`. PNG intensity maps through `height low high`; EXR defaults to metres (`height 0 1`). The same processing runs after this conversion. Without a supplied field, `generate` starts with noise and maps it into `height` instead. Processing order is hydraulic erosion → thermal erosion → terraces → shore → automatic drainage/rivers → biome control. On supplied fields terraces anchor at zero and erosion may leave the original extrema. Author-directed valley lines, river networks and protection masks are separate inputs, not implied by automatic drainage.

## 3. Script, inputs and caching

`og script heights.ts --root assets --params '{"offset":117}' --seed 9 --out og-out/heights.json --json` executes the very same generator as cook. `og mcp` exposes the same `script` tool. `--out` is an explicit JSON output, not permission for the TS module to write files. Run from the recipe project directory; parent directories are created and the complete output is published by rename.

Use repeated `input "name" "relative/path"` entries inside `generate` (or `--input name=path` with script) for source bytes. Only those bytes are exposed at `ctx.inputs[name]`; the script cannot open paths. Parameters, seed, source, transitive imports, declared input contents and relevant engine/toolchain identity determine the shared cook cache key. Moving an unchanged project does not change its key. A source/input change invalidates it even when its timestamp was preserved.

The realm has no filesystem, process, environment, network, system clock or timers. `Math.random` is refused: use an explicitly seeded algorithm and `ctx.seed`. Dynamic code compilation is unavailable; mathematical transcendentals use software libm. Imports are closed before execution. Heap, stack, instruction, module and input budgets stop runaway work. Outputs must be finite synchronous JSON-compatible data; typed arrays become arrays. Invalid source data fails cook before the target asset is published. Runtime handles and world snapshots are not source data.

## 4. Compose the shared mechanisms

`@origingame/engine/generate` exports `procgen` with exactly the signatures documented for `og.procgen`: noise, noiseGrad, curlNoise, worley, noiseGrid, poissonDisk, bsp, wfc, heightmap, plant and fracture. Cook calls the same Rust implementation and IDL parameter reader as the game hosts, without creating a world. An imported generator can also be called by a game: the module then forwards to its normal runtime bindings, as the recipe does above.

Typed buffers remain typed inside the generator, including nested mesh attributes and WFC/BSP grids; a `subarray` denotes only that slice. At the final JSON output boundary they become numeric arrays. Result-returning mechanisms keep `{ok, value}` / `{ok: false, reason}`: check `ok` before composing them. These pure mechanisms do not load assets or return runtime handles. Existing files remain the normal input for plants, sound synthesis and prefracture; calling a mechanism in TS does not bypass a target asset's source validation.

The guide gate writes all attached files and explicitly cooks the project before running the assertions twice. An attached fence's `file=` is project-relative and cannot escape the recipe directory; it belongs to the preceding `ts recipe` main entry. The guide contains no per-recipe shell harness.
