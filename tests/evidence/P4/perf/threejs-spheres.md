# The sphere field against three.js, before and after (owed row 50)

Apple M4 Pro, headless Chrome for Testing 150 (`--use-angle=metal
--enable-unsafe-webgpu --disable-gpu-vsync --disable-frame-rate-limit`),
1280x720 at DPR 1, `bun tools/bench/threejs-compare.ts --frames 600 --warm
60 --rounds 2` (the web-host-tools round's harness and definitions: a frame
is one `frame` call to the engine Worker, one tick and one render;
`sustained` is the timed frames' wall time over their count). Engine: the
3D publish and lanes variants from `OG_RELEASE=1 sh scripts/build.sh`,
served cross-origin isolated (the lanes variant, 8 Workers).

## Before

The web-host-tools round (main at df9c42a, load 41 to 120,
`tests/evidence/P4/web-host-tools/threejs-comparison.md`):

| scene | mode | p50 ms | p95 ms | sustained ms | fps |
|---|---|---|---|---|---|
| spheres | engine WebGPU | 174.69 | 186.837 | 173.872 | 6 |
| spheres | engine WebGL2 | 175.258 | 188.183 | 173.948 | 6 |
| spheres | three WebGL2, InstancedMesh | 2.485 | 4.425 | 2.646 | 378 |
| spheres | three WebGL2, one Mesh each | 12.788 | 15.825 | 13.117 | 76 |
| spheres | three WebGPU, InstancedMesh | 1.533 | 3.013 | 1.65 | 606 |

Reproduced on this branch's base (5dc3aa0) with `tools/bench/profile-spheres.ts`
(`profile-before.txt`): 137.6 ms a frame on WebGPU with the lanes variant,
15.9 ms with the single-threaded wasm, 16.6 ms at 4,000 spheres and 27.7 ms
at 4,100.

## Cause

The engine Worker's CPU profile (CDP Profiler, the publish wasm bound
again without wasm-opt so frames carry Rust names): 85 % of its samples in
`std::sys::sync::condvar::futex::Condvar::wait_timeout` under
`og_lanes::wait_for`, under `og_runtime::extract::rows`. Extraction decoded
every og.Mesh row to a JSON map on the lanes (`og_lanes::map`, 512 rows a
task): on the lanes wasm the eight Workers allocate through one allocator
behind one lock, so the decode serialised on that lock while the engine
thread waited for the lanes. og_lanes splits a call from two grains per
thread's worth of items, which is the step at 4,096 (512 x 8); beyond it the
cost grows with the rows. Single-threaded the same frame was 15.9 ms, most
of it the same JSON maps and the hierarchy's BTreeMaps. Natively (rayon,
the system allocator) the frame was 11.7 ms of CPU, so the defect showed
only on the web.

Fix (dedf817): og.Mesh and og.Transform are read straight from their typed
columns, the hierarchy is dense by entity index and reused across frames,
per-object motion keys are a vector, and rows still decoded as JSON are
decoded serially in the wasm. Every matrix is computed by the same
operations as before (every gfx and template frame unchanged).

## After (bd193d1, merged with main e795264, load 33 to 62)

| scene | mode | p50 ms | p95 ms | sustained ms | fps |
|---|---|---|---|---|---|
| spheres | engine WebGPU | 6.083 | 8.508 | 5.956 | 168 |
| spheres | engine WebGL2 | 5.515 | 10.385 | 5.791 | 173 |
| spheres | three WebGL2, InstancedMesh | 2.71 | 3.893 | 2.778 | 360 |
| spheres | three WebGL2, one Mesh each | 13.24 | 14.168 | 13.289 | 75 |
| spheres | three WebGPU, InstancedMesh | 1.76 | 2.898 | 1.853 | 540 |
| reference | engine WebGPU | 1.01 | 4.098 | 1.793 | 558 |
| reference | engine WebGL2 | 2.44 | 3.863 | 2.372 | 422 |
| reference | three WebGL2, InstancedMesh | 3.738 | 5.17 | 3.644 | 274 |
| reference | three WebGL2, one Mesh each | 4.09 | 5.637 | 3.931 | 254 |

`threejs/threejs-comparison.json` has every run; `threejs/*.png` the
frames (640 px). A single round at load 14 (dedf817) measured the engine's
field at 4.11 ms (WebGPU) and 4.07 ms (WebGL2) against three.js's 2.46 ms
(WebGL2, instanced) and 1.50 ms (WebGPU).

Against design §7 (at least 1.5x three.js's frame rate):

- **Reference scene: met on both tiers now.** WebGPU 558 fps against
  three.js WebGL2's 274 (InstancedMesh, 2.04x) and 254 (one Mesh each,
  2.20x); WebGL2 422 fps (1.54x and 1.66x; the web-host-tools round had
  1.29x against InstancedMesh).
- **Sphere field: 29x faster than before, not yet at three.js.** The
  engine's 168 to 173 fps beat three.js drawing one Mesh per sphere (75 fps,
  2.2x) but not its InstancedMesh: 0.48x of WebGL2's 360 fps and 0.31x of
  WebGPU's 540. The 1.5x target (and parity on WebGL2) is not met.

## What remains, and why

The field is now GPU-bound in the browser: in the engine Worker's profile
after the fix (`profile-after.txt`), 53 % of the busy time is `writeBuffer`
of the instance upload waiting for Chrome's GPU queue; extraction, the
hierarchy and og-render's frame take the rest, about 2.3 ms of CPU a frame
(the tick 0.3 ms). Natively on Metal (`packages/engine/bench/spheres.ts`,
engine.frame then a GPU wait): 1.43 ms of CPU and 5.85 ms with the GPU's
work at 1280x720; 4.59 ms with MSAA off; 3.46 ms at 128x72, where fragments
cost almost nothing, and 8.43 ms at 2560x1440. So about 2 ms of the GPU's
4.4 ms is per-vertex work and the rest per-fragment, against three.js's
whole frame of 1.5 ms (same sphere, MSAA 4x, ACES, one light). The lit
pipeline is the general one: a 64-byte vertex (position, normal, uv, colour,
tangent) and 23 interpolated floats a vertex (clip position, world
position, normal, uv, colour, local position, tangent) for the material
hooks, normal maps and vertex colours, where three.js's
MeshStandardMaterial without maps interpolates about 7, and a fragment
shader with clusters, IBL and the effect hooks. A lean permutation for
materials without maps, vertex colours or hooks (and a thinner vertex
format) is the next step; it changes the lit shaders other streams are
editing this round (pbr_fs.wgsl, physical.wgsl), so it is left as owed
(og-render README, "What phase 2 owes").
