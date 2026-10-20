# The lean lit pipelines (owed row 50's remainder)

Apple M4 Pro, Metal, 1280x720, `aa=Off`, the dev `og-gfx` of this branch
(`cargo build -p og-tests --bin og-gfx`), load average 6 to 10 (other
agents building). Variants, each a copy of the dev binary under
`target/lean/`:

- **base**: main at d08fdc9.
- **keep2**: an intermediate, not committed: the lean vertex stages and
  varyings with the fragment discard kept (what the lean path costs
  without dropping the discard).
- **final**: bddb7411 (the lean lit, prepass and shadow pipelines, the
  discard dropped where no screen derivative is taken).

## Frame times

Five interleaved rounds (base, keep2, final per round), 120 timed frames
after the scene's own, median and [min-max]:

    for r in 1 2 3 4 5; do for v in base keep2 final; do for s in spheres-20k spheres-20k-moving gltf-reference; do
      target/lean/og-gfx-$v $s --frames 120; done; done; done            # og-gfx-overlap.txt
    ... OG_GFX_SERIAL=1 target/lean/og-gfx-$v $s --frames 120            # og-gfx-serial.txt (spheres only)

| scene | variant | frame ms | cpu ms | serial gpu ms |
|---|---|---|---|---|
| spheres-20k | base | 7.115 [6.85-7.23] | 4.270 | 7.341 [7.30-7.92] |
| spheres-20k | keep2 | 6.273 [6.16-6.96] | 4.176 | 6.996 [6.84-7.13] |
| spheres-20k | **final** | **4.585** [4.50-6.92] | 4.116 | 6.095 [4.96-6.32] |
| spheres-20k-moving | base | 7.015 [6.95-7.27] | 4.444 | 7.462 [7.43-7.50] |
| spheres-20k-moving | keep2 | 6.220 [6.12-6.45] | 4.365 | 6.944 [6.81-7.31] |
| spheres-20k-moving | **final** | **4.627** [4.39-5.30] | 3.993 | 6.014 [5.05-6.50] |
| gltf-reference | base | 3.730 [3.62-3.97] | 3.706 | 4.44 (8 runs of 480) |
| gltf-reference | keep2 | 3.692 [3.62-4.02] | 3.676 | |
| gltf-reference | **final** | 3.786 [3.63-4.45] | 3.763 | 4.09 (8 runs of 480) |

The frame stats are unchanged (spheres-20k: draws=10, batches=2,
instances=20001, culled=972, triangles=7306754, shadow_passes=3,
shadow_casters=16719; gltf-reference: draws=113, batches=38,
instances=926, triangles=878827).

- `frame` is the wall time of the timed frames over their count, the
  GPU's work overlapping the next frame's CPU work: it is the GPU's time
  where the GPU is the slower side. The field went from GPU-bound at
  7.1 ms to 4.6 ms, where the dev build's CPU (4.1 ms) is about the
  bound, so the GPU's own time is at most that: **-36 %** or more. Of it,
  the lean vertex stages and varyings alone (keep2) take 0.85 ms
  (-12 %); dropping the never-taken discard, which lets Apple's hidden
  surface removal shade each pixel once over the field's heavy overdraw,
  the rest.
- `gpu` (`OG_GFX_SERIAL=1`, f96b39ee) waits for the GPU after every
  frame: the GPU idles between frames and clocks down, so for a lighter
  frame it overstates the time (final's runs spread from 4.96 to 6.32);
  it is an upper bound, and agrees in direction.
- The §7 reference scene is CPU-bound in the dev build and unchanged
  within noise (3.73 against 3.79 ms overlapped; GPU wait over 8 runs of
  480 frames, `gltf-reference-serial-480.txt`: base median 4.44 ms,
  final 4.09 ms). It does draw lean batches (its untextured parts).

## Same pixels

    target/lean/og-gfx-base  --all --capture target/lean/before   # capture-before.txt
    target/lean/og-gfx-final --all --capture target/lean/after    # capture-after.txt
    for f in target/lean/before/*/*.png; do cmp -s $f target/lean/after/${f#target/lean/before/} || echo DIFF $f; done

All 64 captured frames (`metal`) are byte-identical PNGs between base and
final, and every `vs reference` line is the same as base's: RMSE 0 where
base had 0; the scenes base already differs on against the committed
`metal` references (sprites-20k 0.00003, fx-2d 0.00004, water-3d 0.00062,
terrain-near 0.00657, terrain-mid 0.00362, water-planar 0.00491,
water-shadows 0.00169, water-underwater 0.00042, terrain-grass 0.00495;
max_diff 1 on point-spot-lights and lights-256) differ by exactly as much
after. 45 of the scenes draw lean batches (with effects on in showcase,
overlay, decals, ao-gtao and ao-ssao).

What no committed reference covers was checked A/B the same way with
temporary og-tests scenes (not committed), every one byte-identical:
plain spheres with retro 3D (vertex snap, affine texturing, depth bits)
and pixel scale, with AO, with motion blur (the lean prepass and its
motion clips), with flat shading, and all of them together; and the
showcase without its decals, lights, TAA or bloom.

### Why the discard stays where derivatives are taken

The first cut dropped the discard in every lean pipeline: the showcase and
overlay moved by up to 5/255 on 29 pixels (the scorch decals under the
campfire) and flat-shaded plain spheres by up to 122/255 on 53,000
pixels; with the discard kept (keep2) both were identical. On Metal a
fragment shader with a discard computes screen derivatives differently
from one without, and the full path has one. So a lean pipeline drops it
(`DISCARD` false) only when its fragments take no derivative: not flat
shading (`GpuMaterial::flat`) and no decals in the frame
(`Batch::derivatives`). Skipping the 1x1 white texture samples was tried
too and is not kept: with the metalness and roughness samples gone the
Metal compiler (fast math) rounds the BRDF differently (1/255 on 5 pixels
with AO); the lean fragment samples the white defaults at uv 0 like the
full one, and keeping them cost nothing measurable once the discard was
gone.

## Pipelines and size

Lean lit pipelines are keyed (side, effects, derivatives) and the lean
prepass (side, derivatives), made at first use: a scene of front-sided
plain materials without effects makes one of each; at most 12 and 6. The
shadow pass makes none: its unskinned, undissolved, unhooked casters'
pipeline reads positions alone now (`shadow.wgsl` `vs_lean`/`fs_lean`).
No stylise permutation is needed: the retro clip snap is in the shared
`lit_place`, and the affine uv only feeds maps a plain material does not
have. The palette mode does not apply (the lean path is never skinned).

Shader text in the wasm (`include_str!`): +5,908 bytes raw (pbr_lean.wgsl
3,229, shadow.wgsl +959, pbr_fs.wgsl +884, pbr.wgsl +599,
pbr_prepass.wgsl +237), of which 3,282 bytes are code and the rest
comments; the five files grow by 1,489 bytes under `gzip -9`. The size
gate was not run.

## The instance encoding: not changed

A 3x4 (48 bytes) or position, quaternion and scale encoding would save at
most a quarter of the instance traffic. Natively the whole upload of
20,000 moving transforms (1.28 MB a frame) costs 0.04 ms of frame time
(spheres-20k-moving 4.627 ms against static spheres-20k's 4.585 ms, final;
7.015 against 7.115 base, inside the noise), so a quarter of it is not
measurable here, and the GPU reads an instance's 64 bytes once per vertex
from cache. In the browser the upload's cost was the wait for the busy GPU
queue (`../perf/threejs-spheres.md`), which the lean pipelines address
directly. The change would touch every instance reader (the lit,
physical, shadow, prepass and scatter shaders, GPU culling's compute
stages, `upload.rs`'s dirty blocks, the previous-transform buffer) and the
extraction stream another round is editing, for no pixel or frame it
could show; left as is.
