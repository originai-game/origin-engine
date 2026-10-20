# The sphere field against three.js: instance and vertex encodings, the CPU (owed row 50's remainder)

Apple M4 Pro, Metal, 1280x720, `aa=Off`, branch `render-parity` from main
at 463bc737. The machine was shared all round (other agents' builds, an
iOS simulator drawing continuously): every timing below is interleaved
(base and head alternating round by round) and each round of the final
tables waited for a one-minute load under 8 (`og-gfx-overlap.txt` and
`og-gfx-serial.txt` record it per round). Variants:

- **base**: main at 463bc737; **head**: this branch (5a96a149 and its
  docs). `og-gfx` is og-tests' renderer driver; `-rel` is
  `cargo build --release -p og-tests --bin og-gfx` (the perf lane's
  profile), the others the dev profile the gate runs.
- `frame`: wall time of the timed frames over their count (the GPU's
  work overlapping the next frame's CPU), `cpu`: time inside
  `Renderer::render`, `gpu` (`OG_GFX_SERIAL=1`): the wait for the GPU
  after each frame (an upper bound: the GPU idles and clocks down between
  frames).

No browser was run (the three.js comparison is the coordinator's).

## Where the CPU went (item 1)

macOS `sample` of `og-gfx SCENE --frames 5000`, the main thread's samples
under `Renderer::render` bucketed by what they do (`profile-buckets.txt`;
the bucketing script follows each sample's stack). In release
`build_batches` is inlined into `render` (its share is under "other").

| profile | scene | batches: spheres, GPU transforms, cascade bounds | sort | objc2 verification | encode (wgpu, Metal) | instance upload | submit | other | GPU back-pressure wait |
|---|---|---|---|---|---|---|---|---|---|
| dev, base | spheres-20k | 36.2 % | 9.0 % | 30.2 % | 11.8 % | 4.2 % | 2.6 % | 5.8 % | 0 |
| dev, base | spheres-20k-moving | 35.5 % | 8.3 % | 30.8 % | 10.3 % | 7.0 % | 2.7 % | 5.4 % | 0 |
| dev, head | spheres-20k | 15.5 % | 0.1 % | 0 | 13.1 % | 7.1 % | 3.4 % | 8.3 % | 52.6 % |
| dev, head | spheres-20k-moving | 65.9 % | 0.1 % | 0 | 11.2 % | 11.5 % | 3.2 % | 8.2 % | 0 |
| release, base | spheres-20k | (other) | 0.5 % | 0 | 14.4 % | 4.5 % | 2.7 % | 5.4 % | 72.5 % |
| release, base | spheres-20k-moving | (other) | 0.8 % | 0 | 21.8 % | 14.7 % | 3.6 % | 8.4 % | 50.7 % |
| release, head | spheres-20k | (other) | 0 | 0 | 6.5 % | 2.7 % | 2.1 % | 2.2 % | 86.5 % |
| release, head | spheres-20k-moving | (other) | 0 | 0 | 13.5 % | 13.4 % | 4.0 % | 7.4 % | 61.7 % |

So the dev build's 4.1 to 4.3 ms of CPU a frame (base) was: about 1.5 ms
building the batches (the world bounding spheres, the GPU transforms, the
cascades' bounds over a clone of every sphere), 0.35 to 0.4 ms the sort,
1.3 ms objc2 verifying each Objective-C message's signature (debug
assertions in a dependency: none of it ships), about 0.5 ms recording
wgpu's passes, 0.2 to 0.3 ms the instance upload and 0.1 ms submitting.
The release build spent about 1.6 ms a frame in `render`, most of it
waiting on Metal's command-buffer pool (the GPU is the slower side: the
frame is GPU-bound natively), the rest wgpu's recording (including its
indirect-draw validation pass, `inject_validation_pass`), the upload
(`write_buffer`'s copy and the staging buffer it allocates each call) and
submit. Extraction (og-runtime, outside og-gfx): `extract_cost`'s
`perf_extract_spheres_20k` measures 0.437 ms a frame in release against
its 0.531 ms baseline (`extract-release.txt`; the test's own note puts the
dev profile at 3.8 ms).

## What changed

| commit | change |
|---|---|
| 038cc380 | The sorted instances are kept across frames (`instances.rs`): the sort is redone only when the submission's (mesh, material) sequence or a drawn mesh's bounds changed; an instance's sphere and GPU transform only when its transform changed (bit for bit); the cascades' automatic bounds fold the kept spheres' box with the skinned instances and terrains in the old order (no clone). |
| 8f4468f2 | Instances are 48 bytes on the GPU: the x, y, z of an affine matrix's four columns (`instances::GpuTransform`), in every instance buffer (CPU path, GPU culling's regions, previous transforms, skinned instances, scattered copies), rebuilt by `instance.wgsl`'s `instance_matrix`. |
| 8d619852 | The compact vertex encoding, 16 bytes, opt-in per mesh (`RenderScene::set_vertex_encoding`, `compact.rs`): position `Unorm16x4` over the bounds, normal octahedral `Snorm16x2`, colour `Unorm8x4`, the dequantisation folded into the instance transforms of the runs that draw it; compact variants of the lean lit, lean prepass, lean shadow and occluder-depth pipelines; the f32 vertices uploaded only when something else draws the mesh. |
| 07902610 | No wgpu indirect-call validation (GPU culling's draws are in range by construction). |
| cbcc062c | objc2 without debug assertions in the dev profile (Cargo.toml). |
| 5a96a149 | The kept instances' per-frame pass made cheaper at opt-level 0 (one pass for handles and keys, no array maps). |
| 6f8eee08 | Merge of main (d243e461): main's array maps carried an instance's texture-array layer in the 4x4's m0.w, which the columns encoding drops, so the GPU instance became the columns and the layer, **52 bytes** (location 8, read by `pbr_array.wgsl` alone; 13 floats a slot in `cull.wgsl`). The timings below were taken at 48 bytes, before the merge. |

### Same pixels

    target/parity/og-gfx-base --all --capture before    # capture-before.txt
    target/parity/og-gfx-head --all --capture after     # capture-after.txt
    for f in before/metal/*.png; do cmp -s $f after/metal/${f#before/metal/} || echo DIFF $f; done

All 68 Metal frames are byte-identical PNGs between base and head, and
every stats and `vs reference` line is the same (setup times aside); the
check was repeated after each commit. The 48-byte instance was not so at
first: rebuilt with a literal fourth row (0, 0, 0, 1), 45 of the 68
frames moved (rmse up to 0.006, edge pixels flipping by up to 164/255).
Bisecting the shader (experiments with the old 64-byte buffer and only
the matrix's construction changed) showed the cause: the world point's w
became a constant the Metal compiler can see, and it then reorders the
sum of the projection that follows (the w term first); a world point
whose xyz came from the rebuilt matrix and whose w came from the buffer
gave the old pixels, and so did an explicit column-order fused chain
(`fma(m[3], w, fma(m[2], z, fma(m[1], y, m[0] * x)))`) with the literal
w, while three other orders did not. `instance.wgsl`'s `mul_point` is that
chain, used for the projections of instance-moved points (view, shadow
casters, previous view, occluders, GPU culling's screen boxes); a point
moved by a material's `vertex_offset` keeps the plain product, which the
old buffer-borne w did not order (`hooks.rs` `moved_clip`). The GL and
SPIR-V tiers compile the same arithmetic (GLSL ES 3.0 writes the fma as
`a * b + c`); not run here (no lavapipe or WebGL2 on this machine), their
references are compared under the gate's tolerance.

After the merge, against main at d243e461 (`capture-main-d243e461.txt`,
`capture-merged.txt`, 73 frames): 71 byte-identical with identical stats
lines; `stylise-object` now equals its committed reference byte for byte
(main's own frame is 135 pixels off it by 1/255, since main zeroed m0.w in
the object pass), and `materials-array` moves 13 pixels by 1/255 (rmse
0.00001, inside every tolerance) from the reference main recorded for it:
main's array stages built the matrix with a literal 0 in m0.w and the
other w from the buffer, an arithmetic the 52-byte instance no longer
has.

With `OG_GFX_COMPACT=1` (every mesh compact) all 68 scenes draw
(`capture-compact.txt`); against main's frames the largest difference is
showcase's rmse 0.011 (lighting dither around the campfire light from the
ground's quantised normals), spheres-20k rmse 0.0059.

## Frame times (item 4)

7 interleaved rounds of 120 frames (`og-gfx-overlap.txt`), median [min-max]:

| scene | variant | frame ms | cpu ms (median [min]) |
|---|---|---|---|
| spheres-20k | base | 5.636 [5.59-5.69] | 4.265 [3.990] |
| spheres-20k | **head** | **5.392** [5.32-5.92] | **1.757** [1.628] |
| spheres-20k | base-rel | 5.676 [5.44-5.84] | 1.602 [1.511] |
| spheres-20k | **head-rel** | **5.492** [5.37-5.80] | **1.129** [1.082] |
| spheres-20k-moving | base | 5.674 [5.59-5.88] | 4.266 [4.112] |
| spheres-20k-moving | **head** | **5.537** [5.39-5.80] | **2.964** [2.834] |
| spheres-20k-moving | base-rel | 5.722 [5.58-6.11] | 1.712 [1.565] |
| spheres-20k-moving | **head-rel** | **5.534** [5.39-5.99] | **1.272** [1.202] |
| gltf-reference | base | 3.922 [3.77-4.03] | 3.889 [3.753] |
| gltf-reference | **head** | **1.960** [1.84-2.01] | **0.945** [0.900] |
| gltf-reference | base-rel | 1.871 [1.83-2.03] | 0.642 [0.617] |
| gltf-reference | **head-rel** | **1.881** [1.80-1.94] | **0.618** [0.590] |

4 interleaved rounds of 240 frames with a GPU wait after each
(`og-gfx-serial.txt`; here `cpu` is the CPU alone):

| scene | variant | cpu ms | gpu ms |
|---|---|---|---|
| spheres-20k | base-rel / head-rel | 0.805 / 0.764 (min 0.759 / 0.530) | 6.825 / 6.896 |
| spheres-20k-moving | base-rel / head-rel | 0.918 / 0.968 (min 0.814 / 0.725) | 6.795 / 6.882 |
| gltf-reference | base-rel / head-rel | 0.389 / 0.432 (min 0.361 / 0.358) | 2.898 / 2.820 |

- **The dev build's CPU**: spheres-20k 4.27 to 1.76 ms (-59 %),
  spheres-20k-moving 4.27 to 2.96 ms (-31 %), gltf-reference 3.89 to 0.95
  ms (-76 %). In the still field the sort and the spheres are gone (the
  frame waits on the GPU now); in the moving one every sphere and
  transform is recomputed, as before, so its gain is objc2's. The dev
  frame of gltf-reference halves (3.92 to 1.96 ms): it was CPU-bound.
- **Release CPU** in `render`: spheres-20k 1.60 to 1.13 ms, moving 1.71
  to 1.27 ms (the indirect validation's pass and the smaller upload); the
  serial runs' CPU (no waits) is within noise of base's at 0.5 to 1 ms.
- **The frame** is GPU-bound on both, at 5.4 to 5.5 ms against 5.6 to 5.7
  (about -3 %, the validation pass the GPU no longer runs); the GPU's own
  time (serial) did not move. gltf-reference in release is unchanged.
- **The §7 reference scene** natively (`packages/engine/bench/lanes.ts
  --native --lanes on --frames 600`, release addons of base and head,
  4 interleaved rounds, `reference-native.txt`): p50 4.71 to 4.89 ms base,
  4.57 to 5.06 ms head, CPU p50 0.95 against 0.95 ms; the same world hash
  (37fb8900fad8462f). Unchanged: its 500 objects take the CPU path and
  turn every frame.

### Perf baselines (lowered)

The perf test's method (`crates/og-tests/tests/perf.rs`: 7 runs of 100
frames, the best) with the release og-gfx (`perf-method.txt`; the perf
lane itself was not run): spheres-20k best 5.245 ms (base 5.540),
spheres-20k-moving 5.276 ms (base 5.675), gltf-reference 1.739 ms.
`crates/og-tests/perf/metal-apple-m4-pro/spheres-20k.json` 6.675 to 5.245
and `spheres-20k-moving.json` 7.759 to 5.276; gltf-reference's 1.385 ms
was not beaten on this machine today and stays.

## Where the GPU goes, and why the encodings do not move it natively

Release og-gfx, spheres-20k(-moving), serial GPU time, 2 to 3 rounds each:

| experiment | gpu ms |
|---|---|
| as drawn (3 shadow cascades) | 6.3 to 6.6 |
| no shadows | 3.9 to 4.5 |
| no shadows, every sphere scaled to 1/20 (almost no fragments) | 3.9 to 4.4 |
| no shadows, lean vertex stage without the colour varying | 4.0 to 4.1 (against 4.1 to 4.5) |
| no shadows, normals through the model matrix instead of the cofactor | 4.2 (against 4.1 to 4.5) |
| no shadows, compact vertices (`OG_GFX_COMPACT=1`), MSAA 4x | 4.1 to 4.5 (against 4.0 to 4.5) |
| with shadows, compact vertices | 6.2 to 6.8 (against 6.3 to 6.6) |
| no shadows, 20,000 / 10,000 / 5,000 instances (overlapped frame) | 3.3 to 3.7 / 1.6 to 1.7 / 1.0 |

The field's GPU time follows its instance count and not its pixels, its
varyings, its vertex stage's arithmetic or its vertex fetch: it is bound
by the 7.3 million triangles a pass (4.4 million vertex invocations),
which the 48-byte instance and the 16-byte vertex do not reduce (the
sphere's 221 vertices are 14 KB, in cache whatever their size). 16-bit
index buffers were tried too (half the index bytes): 5.36 to 5.50 ms
against 5.46 to 5.70, inside the noise, not kept. The browser's case may
differ (Chrome's GPU queue and `writeBuffer` were the bound there,
`../perf/threejs-spheres.md`); the 25 % smaller upload and the compact
encoding are for that comparison to show.

## Not done

- The compact encoding is not switched on anywhere by default: it is
  quantised, so it would move the frames of every plain mesh. Cooked
  models (og-cook `CookedModel::add_to_with`) could set it for their
  primitives without tangents in one line; that moves the playtest and
  cooked-model frames of templates with untextured cooked parts (within
  tolerance, by quantisation), so it is left to a round that re-records
  them.
- design §7 is unchanged: natively the field is GPU-bound at about 5.5 ms
  a frame with shadows and 3.3 to 4.5 ms without, against three.js's
  1.65 ms in Chrome; the target is not met natively.
