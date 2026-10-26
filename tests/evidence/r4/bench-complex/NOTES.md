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

## r4/bench-complex-2: two defects of the three.js side

Branch `r4/bench-complex-2` from `58f5609e` (the merge of `r4/bench-complex`).

The Mac lane's first run (`threejs-compare.ts --only complex --rounds 2`,
Chrome 150, M4 Pro), sustained ms:

| mode | ms |
|---|---|
| engine WebGPU | 5.658 |
| engine WebGL2 | 7.587 |
| three WebGL2, InstancedMesh per tile | 10.349 |
| three WebGL2, InstancedMesh per part | 24.451 |
| three WebGPU, InstancedMesh per tile | 30.45, of a black canvas: void |

The verdict was inconclusive for two reasons, both on three's side.

**1. three's WebGPU canvas was black (Mac and orb).** Chrome logs, for every
lit material: "The number of sampled textures (17) in the Fragment stage
exceeds the maximum per-stage limit (16)". three 0.180's `ShadowNode`
samples a colour map (the tint of transparent casters) beside each shadowed
light's depth map. With the sun's three cascades and four local lights, that
is 7 depth maps + 7 colour maps + base colour + normal map + environment:
17 textures and 17 samplers in one stage. Every pipeline was refused, so
nothing drew; the 30 ms were three's CPU work over invalid pipelines.
Chrome raises `maxSampledTexturesPerShaderStage` to 48 on request (Chrome
146), but `maxSamplersPerShaderStage` stays 16, so `requiredLimits` cannot
fix it. The fix: `three-complex.js` replaces `ShadowNode.prototype.setupShadow`
with three's own PCF path minus the colour sample. It is built from the
node's own `setupRenderTarget`, `setupShadowCoord` and `setupShadowFilter`,
so the bias and the filter are three's. The output is `mix(1, shadow,
intensity)`, which is what opaque casters gave before (they write (0, 0, 0,
1) into the colour map). The depth test alone is what WebGLRenderer and the
engine do; three does less work than before, not more. The villager
body's fragment shader (dumped in the orb with `off=props,post`) went from
12 textures and 12 samplers to 8 and 8. `__bench_info__.sampledTexturesPerStage` reports the
device's limit.

In the orb, three's WebGPU canvas also needs `--use-angle=gl-egl`; without it
Chrome cannot create the swap chain's shared image and the device is lost
("Device was destroyed"). The harness passes it through `BENCH_CHROME_FLAGS`.
That is why the first round's orb shot of three WebGPU was blank. The Mac's
Metal adapter does not need it.

**2. three's WebGL2 frames were washed out, without the sun's shadows.** CSM
worked: three 2048 cascades, `castShadow` on 926 of 933 meshes, receive on 931.
Turning casting off changed about 89k pixels. The shadows were there but
drowned: three's `Sky` (and `SkyMesh` on WebGPU) is a much brighter
atmosphere than the engine's, and it lit the scene through PMREM at
intensity 1. The fix: three gets the engine's own analytic sky. That is
`fs_sky` of `crates/og-render/shaders/ibl.wgsl`: zenith, horizon, ground,
the sun's glow (pow 24 × 0.6) and disc (smoothstep 0.9993–0.9997 × 40),
× 1.1. It is drawn once on the CPU into a 512 × 256 half-float equirect and
prefiltered by `PMREMGenerator.fromEquirectangular`. It lights the scene at
the engine's `RenderSettings.environmentIntensity` (0.55), and the same
texture is the background. Tone mapping was already ACES at exposure 1 on
both sides and is unchanged.

Diagnostic switches (never for timing): `three-page.html?scene=complex&off=`
takes a comma list of `props`, `chars`, `csm`, `post`, `traa`, `aobloom` and
`normalmaps`. `globalThis.__three__` exposes the scene, camera, renderer and
CSM after the first frame.

### The orb's receipts, round 2

In `orb-2/` (`threejs-comparison.json`, `stdout.txt`, `stderr.txt`, the shots as JPEG):

`BENCH_CHROME_FLAGS="--use-angle=gl-egl" bun tools/bench/threejs-compare.ts --only complex --frames 3 --warm 1 --rounds 1 --shot-frames 2 --shot-wait 1500 --out <dir>`

| mode | p50 ms | p95 ms | sustained ms |
|---|---|---|---|
| engine WebGPU | 30.04 | 599.5 | 218.1 |
| engine WebGL2 | 2,758.4 | 5,831.4 | 3,776.7 |
| three WebGL2, InstancedMesh per tile | 2,648.7 | 2,739.7 | 2,647.9 |
| three WebGL2, InstancedMesh per part | 20.5 | 21.6 | 20.8 |
| three WebGPU, InstancedMesh per tile | 215,907 | 491,186 | 244,783 |

These are a CPU rasteriser's numbers, so no ratio is read from them. The one
that does say something: three WebGPU now takes about 216 s per frame on
SwiftShader, where the black canvas took 58 s in round 1. Its pipelines now
run; before, they were refused. Chrome's log for the round-2 page has no
WebGPU validation error.

The patch was also checked against stock three on a minimal WebGPU scene: a
ground plane, nine boxes, the sun through `CSMShadowNode` with the same
settings. The frames with and without the patch differ in 0.45 % of pixels
(filter noise), and the cast shadows are identical.

The shots, the same camera on all sides:

- `threejs-complex-engine-webgpu.jpg`, `…-engine-webgl2.jpg`,
  `…-three-webgl2-instancedmesh-per-tile.jpg`, `…-per-part.jpg`: the same
  village, the sun's shadows cast by the houses, stalls and trees on all
  four, and saturation and exposure close to each other. The engine's lamp
  glass blooms more.
- `three-webgpu-two-frames-no-keepalive.jpg`: three WebGPU two frames after
  load (`?scene=complex&webgpu=1`). The same framing and content, with the
  sun's shadows. It is a little darker and more contrasted than three
  WebGL2. The white strips on the roofs (the gables' wall boxes between the
  roof slabs) show less, which is a depth-resolution difference, not
  lighting.
- `threejs-complex-three-webgpu-instancedmesh-per-tile.jpg`: the harness's
  shot of three WebGPU. The harness keeps stepping every 30 ms while it waits
  for the screenshot, and on SwiftShader that wait lasted about 25 minutes,
  so the orbit has moved on (a wider view). TRAA, fed frames whose real
  delta is minutes, ghosts: the pine tips are red and the roofs are washed
  out. On the Mac the same harness plays 120 frames at full rate; its shot
  should be checked for the same ghosting, which the orb cannot settle.

### What the Mac runs (round 2)

From a checkout of `r4/bench-complex-2`, as before (`packages/engine/dist`
built, `bun install --cwd tools/bench --frozen-lockfile`):

```sh
bun tools/bench/threejs-compare.ts --only complex --rounds 2 --out tests/evidence/r4/bench-complex/mac-2
```

Before reading a ratio, check that `threejs-complex-three-webgpu-instancedmesh-per-tile.png`
is not black and shows the sun's shadows, and that `threejs-comparison.json`
for three WebGPU has no console errors. Its `info.sampledTexturesPerStage`
reports the device's limit.

## Mac-2 numbers

Measured at `303763267e88c8363f4c6945d626efc8a9a0f9da` on the Mac's M4 Pro,
Chrome for Testing 150.0.7871.115, with the current revision's wasm rebuilt
before the package build. Command: the round-2 command above. The canvas
was 1280 × 720 at DPR 1, with 60 warm frames and 600 timed frames per mode,
two rounds interleaved by mode. `OG_PERF_RECORD` was unset.

| Mode | Round 1 sustained ms | Round 2 sustained ms | Mean ms | FPS |
|---|---:|---:|---:|---:|
| Engine WebGPU | 5.687 | 5.709 | 5.698 | 176 |
| Engine WebGL2 | 7.862 | 7.860 | 7.861 | 127 |
| Three WebGPU, per tile | 31.161 | 30.939 | 31.050 | 32 |
| Three WebGL2, per tile | 9.740 | 9.892 | 9.816 | 102 |
| Three WebGL2, per part | 24.914 | 24.946 | 24.930 | 40 |

The two-round mean is also the median of these two observations. Ratios
use sustained time, not rounded FPS:

- **WebGPU primary: 31.050 / 5.698 = 5.449×, PASS** against §7's ≥ 1.5×.
- WebGL2 per tile: 9.816 / 7.861 = **1.249×**, reported beside the judge.
- WebGL2 per part: 24.930 / 7.861 = **3.171×**.

The visual prerequisites were checked before interpreting the ratios:

- Three WebGPU is not black. The capture shows the village and the sun's
  cast shadows. Pine tips remain green and roofs retain their red/brown
  detail, with no visible red-tip or washed-roof TRAA ghosting.
- Three WebGL2 per tile now shows directional house and tree shadows,
  with the same level and camera framing as the engine. The engine's
  cyan emissives still bloom more strongly.
- `info.sampledTexturesPerStage` is **16** in both three WebGPU rounds.
- All ten timed runs completed. The JSON has no error results, and the
  captured stderr has no console errors or exceptions. The harness does
  not serialize a console-errors field into its JSON; stderr is the
  console-error receipt.

Load caveat: the benchmark followed a 60-second pause after the build.
The one-minute load at each timed run's start was 6.75–7.88, below 8,
but the five- and fifteen-minute averages remained elevated from the
preceding build. These are two rounds, not a broader variance study.

Receipts are in `mac-2/`: `threejs-comparison.json`, `stdout.txt`,
`stderr.txt`, and the five PNGs. No baseline or reference was re-recorded.

## Mac-3 numbers (vendored GLES patches)

Measured at `9498bc0a48846ae38a54256e7bfbeaf8b5d1d0a5`, M4 Pro,
Chrome for Testing 150.0.7871.115. Release addon and wasm were rebuilt;
the gate wasm was also refreshed with `OG_WASM_PROFILE=wasm` before the
browser lane copied its artifacts. The benchmark uses publish wasm with
GPU timers disabled. Command:

```sh
bun tools/bench/threejs-compare.ts --only complex --rounds 2 --out tests/evidence/r4/bench-complex/mac-3
```

Same 1280×720, 60 warm frames, 600 measured frames and two interleaved
rounds. `OG_PERF_RECORD` was unset; a 60-second pause preceded the benchmark.
Run-start one-minute load samples were 3.71–5.62 (five-minute 4.82–5.18,
fifteen-minute 6.13–6.17). No continuous idle-load claim is made.

| Mode | Round 1 sustained ms | Round 2 sustained ms | Mean ms | FPS |
|---|---:|---:|---:|---:|
| Engine WebGPU | 5.347 | 5.108 | 5.228 | 191 |
| Engine WebGL2 | 7.773 | 7.410 | 7.592 | 132 |
| Three WebGPU, per tile | 30.296 | 30.744 | 30.520 | 33 |
| Three WebGL2, per tile | 9.805 | 10.162 | 9.984 | 100 |
| Three WebGL2, per part | 24.404 | 24.075 | 24.240 | 41 |

The mean is also the median of two runs. WebGPU per-tile / engine is
**5.838×, PASS** against §7's ≥1.5×; WebGL2 per-tile is **1.315×** and
per-part **3.193×**, reported. WebGL2 engine time is 7.592 ms versus
Mac-2's 7.861 ms (0.269 ms, 3.42% lower). That difference is smaller than
the current two-run range of 0.363 ms: these receipts do not isolate a
causal speedup from the state-cache patch or distinguish it from build,
load and run variance. The separate `gl-calls.ts` check does establish
1,345 calls/frame, below its 1,500 bound.

The screenshots were inspected before interpreting ratios. Three WebGPU
renders the village, green pine tips and detailed roofs, without red-tip
TRAA ghosting; its PNG is byte-identical to the accepted Mac-2 capture
(SHA-256 `68e579ae6ff855f2193c7901d3c9ca5035f29697f13aa662b675c31da9d350c8`).
Its shadow contrast remains weaker than WebGL2, unchanged from Mac-2.
Three WebGL2 shows the sun's cast house/tree shadows; its level and camera
match the engine, whose cyan emissives bloom more strongly. Both three
WebGPU rounds report `info.sampledTexturesPerStage=16`. All ten timed
runs completed, no result contains an error, and stderr contains no
console-error or exception report. JSON, stdout/stderr and five PNGs are
retained in `mac-3/`. No source, baseline or reference was changed.
