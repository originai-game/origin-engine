# r4/bench-complex: the complex scene, the judge of §7's 1.5×

The owner's decision of 2026-10-25: the three.js budget of docs/design.md
§7 (≥ 1.5× the frame rate of the same scene in three.js) is judged on a
product-scale scene, WebGPU the primary tier and WebGL2 reported, because
the §7 reference scene (500 objects, 20 walkers) is small enough to hide
the gap between the engines: on the Mac (M4 Pro, Chrome) it measured
1.93× on WebGPU and 1.266× on WebGL2. The reference scene stays, reported
beside it.

Branch `r4/bench-complex` from `6ac9563d`. No Rust changed; the scene is
TypeScript and assets made by code, the three.js side is JavaScript.

## What the scene contains

`tools/bench/complex` (the engine project) and `tools/bench/three-complex.js`
(three.js 0.180). Both place everything from `complex/src/layout.js` (a
seeded hash, no clock) and load the same files from `complex/assets/`
(written by `complex/tools/make-assets.ts`; not committed, made and cooked
by `threejs-compare.ts` when the generator or the layout changed).

| | count |
|---|---|
| ground | one 320 m model, 256 × 256 quads, 131,072 triangles, four textured materials (grass, dirt, rock, sand) with a normal map; a lake under a blended water plane |
| props | 5,261 copies of 19 models, 80 mesh parts, 60 materials, 3,033,128 triangles drawn at full detail |
| by model | 1,100 pines, 700 oaks, 300 + 300 + 300 rocks (three shapes, one mossy), 900 bushes, 300 berry shrubs, 180 palms, 24 islets, 260 crates, 240 barrels, 420 fence panels (14 paddocks), 96 lamp posts (emissive glass), 36 houses (5 materials, blended windows), 32 market stalls, 40 cars, 6 stairs, 3 dropships (16 materials, blended cockpit), 24 ghost carts (blended, moved every tick) |
| skinned characters | 48: 24 walkers (12 joints; idle, walk, run, wave, turn) and 24 villagers (21 joints; kestrel-stair's library: idle, walk, run, attack, hurt, jump, bite, slam, die), 14 (model, clip) pairs, each its own speed and phase |
| lights | the sun, three 2048 cascades over 120 m; 2 spot and 2 point lights, all shadowed (512, cube faces) |
| particles | 20 emitters: 4 fires (additive) and 16 smokes (4 campfires, 12 chimneys; blended), ≤ 64 particles each |
| post | AO, bloom, ACES, the tier's antialiasing (TAA on WebGPU, MSAA 4× on WebGL2) |
| camera | a slow orbit over the village, 55° fov, near 0.3, far 600, 1280 × 720 at DPR 1 |

Assets reused from the engine (copied, byte-identical): `walker.glb`
(reference scene, CC0), `palm`, `berry`, `islet` (rumpus-isle, CC0), `car`,
`ghost` (meridian-loop, CC0), `stair` (kestrel-stair, CC0), `dropship`
(templates/game, MIT). The rest (ground, trees, rocks, bush, crate, barrel,
fence, lamp post, house, stall, villager, particle textures) is made by
`make-assets.ts` from kestrel-stair's glTF, media and rig writers, seeded.

Determinism: `og run tools/bench/complex --headless --ticks 120` twice
(`OG_RENDERER=none`): `tick 120 hash 814bc0dd60cc490f entities 5336` both
times.

## How the two sides are matched

The full table is in `tools/bench/README.md` §The complex scene. In short:

- the same `.glb` files on both sides; three samples the engine's own cooked
  KTX2 (ETC1S colour, UASTC normal and linear) read out of `cooked/*.ogb`
  through `KTX2Loader`, so both sample the same compressed texels;
- the engine draws at full detail (`RenderSettings.lodBias 0`), since three
  has no LOD;
- three instances every (mesh part, material) as an `InstancedMesh` per
  40 m tile (878 instanced meshes, so its frustum culling still culls),
  and the per-part variant (one `InstancedMesh` per part over the level) is
  timed beside it; the ghost carts' matrices are rewritten every frame;
- `SkinnedMesh` per character (`SkeletonUtils.clone`), an `AnimationMixer`
  each, the same clips, speeds and phases;
- the sun through `CSM` / `CSMShadowNode` (three cascades of 2048 over
  120 m, practical split), four shadowed `SpotLight`/`PointLight` at 512;
- the same emitters simulated on the CPU from the same table, one
  instanced billboard draw per blend mode;
- post on WebGL2: `GTAOPass`, `UnrealBloomPass`, `SMAAPass`, `OutputPass`
  (ACES); on WebGPU: `GTAONode`, `BloomNode`, `TRAANode`, ACES.

What cannot be the same, and why it does not favour the engine:

- **Antialiasing on WebGL2**: the engine's tier draws MSAA 4×; three's
  TAARenderPass re-renders the scene per sample on a moving scene, so three
  takes SMAA, a single post pass and the cheaper of the two. On WebGPU both
  are temporal (TAA and `TRAANode`).
- **AO on WebGL2**: the engine's tier runs SSAO; three runs GTAO on both
  tiers, the costlier one.
- **Clustered lights**: the engine culls lights per cluster (forward+);
  three shades all four in every fragment. It is the engine's design, not a
  switch; with four local lights its effect is small.
- **Culling grain**: the engine culls per instance; three culls per
  `InstancedMesh`, hence the tiles, the faster of the two three variants.
- **Particles**: three has no particle system; its simulation here is the
  simplest that reproduces the emitters (no noise, no soft particles, which
  the emitters do not ask for), less work than the engine's.
- **WebGPU skinning in three 0.180**: `SkinningNode` is shared by clones of
  one material and writes `skeleton.previousBoneMatrices` (for TRAA's
  velocity) of every clone once one has it, so each clone is given its own
  array (`three-complex.js`); without it the page throws on the second
  character.

## The orb's receipts

An orb has no GPU (Chrome 156 headless; WebGL2 through
`--use-angle=gl-egl`, Mesa llvmpipe, as `tests/evidence/r4/perf-webgl2`,
since SwiftShader refuses the engine's msdf-text pipeline; WebGPU on
SwiftShader). The numbers prove that every mode runs to completion and
reports; they say nothing of the ratio. The Mac lane measures.

`BENCH_CHROME_FLAGS="--use-angle=gl-egl" bun tools/bench/threejs-compare.ts --only complex --frames 10 --warm 3 --rounds 1 --out /tmp/bc/run2`:

| mode | p50 ms | p95 ms | sustained ms |
|---|---|---|---|
| engine WebGPU | 19.43 | 51.88 | 25.05 |
| engine WebGL2 | 2,855.9 | 6,248.7 | 3,001.8 |
| three WebGL2, InstancedMesh per tile | 2,624.0 | 2,826.8 | 2,629.5 |
| three WebGL2, InstancedMesh per part | 13,757.3 | 14,376.0 | 11,426.1 |
| three WebGPU, InstancedMesh per tile | 73,792.5 | 78,327.0 | 72,560.9 |

All five modes started, ran their frames and answered (the stderr lines of
the harness, `p50 … sustained …`); the run was stopped in its screenshot
phase, whose 120 frames take hours on SwiftShader WebGPU, and repeated with
fewer frames for the JSON and the screenshots (below). On a software
adapter the backends differ by orders of magnitude for reasons that have
nothing to do with either engine (the engine's WebGPU frames ran at 25 ms
while its own WebGL2 frames took 3 s), so no ratio is read from this
table.

Run 3, the whole harness to its end (timing, screenshots, the JSON), in
`orb/` (`threejs-comparison.json`, `stdout.txt`, `stderr.txt`, the shots as
JPEG):

`BENCH_CHROME_FLAGS="--use-angle=gl-egl" bun tools/bench/threejs-compare.ts --only complex --frames 3 --warm 1 --rounds 1 --shot-frames 2 --shot-wait 900 --out <dir>`

| mode | p50 ms | p95 ms | sustained ms |
|---|---|---|---|
| engine WebGPU | 33.67 | 486.3 | 179.5 |
| engine WebGL2 | 2,963.0 | 6,125.5 | 3,948.9 |
| three WebGL2, InstancedMesh per tile | 2,801.3 | 3,017.8 | 2,842.2 |
| three WebGL2, InstancedMesh per part | 33.4 | 33.6 | 23.6 |
| three WebGPU, InstancedMesh per tile | 58,485.1 | 148,452.7 | 73,948.5 |

(Three frames a mode: the per-part figure here, against 11.4 s a frame in
run 2, shows how little a CPU rasteriser's numbers mean.)

The screenshots (two frames after load, the same camera on both sides):
`orb/threejs-complex-engine-webgpu.jpg`, `…-engine-webgl2.jpg`,
`…-three-webgl2-instancedmesh-per-tile.jpg`, `…-per-part.jpg`. Both pages
draw the same level: the village's two house rings round the market, the
crates and barrels, the lamp posts, the ghost carts on the ring road, the
characters on the plaza, the forest, rocks and berry shrubs, the lake with
its palms at the left, the cars at the right edge. The lighting is not
tuned to match pixel for pixel: three's PMREM sky lights the scene more
brightly and its frame is hazier, the engine's is more saturated, as on
the reference scene. three's WebGPU shot is blank on the orb (a WebGPU
canvas of three's does not reach `Page.captureScreenshot` there; the
engine's does); the Mac's run captures it.

## What the Mac runs

From a checkout of `r4/bench-complex`, with `packages/engine/dist` built
(`sh packages/engine/scripts/build.sh`, release or the gate's) and
`bun install --cwd tools/bench --frozen-lockfile`:

```sh
# the judge: the complex scene, every mode, two rounds, then the screenshots
bun tools/bench/threejs-compare.ts --only complex --rounds 2 --out tests/evidence/r4/bench-complex/mac
# the reference scene, reported beside it
bun tools/bench/threejs-compare.ts --only reference --rounds 2 --out tests/evidence/r4/bench-complex/mac-reference
```

The ratio that judges §7 is `three WebGPU, InstancedMesh per tile`'s
sustained ms over `engine WebGPU`'s (the faster three variant on the same
tier); WebGL2's is `three WebGL2, InstancedMesh per tile` over
`engine WebGL2`. The first run makes and cooks the assets (about a second
and three; `og cook` from the gate's or the package's `og`).
