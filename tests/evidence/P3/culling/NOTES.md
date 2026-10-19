# GPU-driven culling: evidence

Recorded 2026-10-19 on an Apple M4 Pro (macOS, Metal 3), Google Chrome for
Testing 150 headless, on branch p3-culling. The scenes are og-tests'
perf-only city scenes (`crates/og-tests/src/culling_scenes.rs`, 1f94224):
40,000 instanced buildings in 16 batches at 1280x720. `city-street` walks an
avenue where the first rows hide most of the city; `city-aerial` sees it all
from above. Neither has a reference; they exist to be timed.

The three variants:

- **before**: `og-gfx` built at 1f94224, the commit before GPU culling
  (7d845e1). Kept as `target/evidence/culling/og-gfx-before`.
- **gpu**: `og-gfx` on this branch, `Culling::Auto`, which takes the GPU
  path on Metal and on WebGPU (the city's 40,000 instances are above the
  later threshold, so `Auto` still does; `OG_GFX_CULLING=gpu` forces it).
- **cpu**: the same binary with `OG_GFX_CULLING=cpu` (`Culling::Cpu`, the
  WebGL2 path), or `--cpu-culling` in the browser.

The machine was shared with other agents' gates throughout, at load averages
between 25 and 200. Every native figure therefore comes from rounds that
interleave the variants and is read as a median or as a ratio within a round.
A single run means little.

## Native Metal

    cargo build --release -p og-tests --bin og-gfx          # after (this branch)
    # before: the same command at 1f94224, copied to target/evidence/culling/og-gfx-before
    target/evidence/culling/og-gfx-before city-street --frames 300
    target/release/og-gfx city-street --frames 300
    OG_GFX_CULLING=cpu target/release/og-gfx city-street --frames 300

`metal/interleave.sh ROUNDS FRAMES OUT` runs the three commands for both
scenes once per round. The rounds were `5 300` (`metal/rounds-a.txt`),
`15 300` (`rounds-b.txt`) and `10 300` twice (`rounds-c.txt`, `rounds-d.txt`),
40 rounds in all. `python3 metal/summarize.py metal/rounds-*.txt` prints the
tables below.

The measure is og-gfx's `cpu=`: the wall time of `Renderer::render` per frame
(encode, cull, upload, submit), averaged over the 300 timed frames. `frame=` adds
the wait for the GPU at the end.

`Renderer::render`, ms per frame, 40 rounds:

| Scene | before median | gpu median | cpu median | before min | gpu min | cpu min |
|---|---|---|---|---|---|---|
| city-street | 6.28 | 5.27 | 6.40 | 2.64 | 2.13 | 3.52 |
| city-aerial | 5.09 | 5.13 | 5.87 | 2.03 | 2.33 | 2.13 |

Median of the 40 ratios taken within a round:

| Scene | gpu / before | cpu / before | gpu / cpu |
|---|---|---|---|
| city-street | 0.81 | 0.92 | 0.83 |
| city-aerial | 0.97 | 1.05 | 0.86 |

Process CPU time per frame counts every thread, including Metal's driver
threads. It is measured as the difference of `/usr/bin/time -p` user+sys
between a 60-frame run and a 660-frame run: `metal/cputime.sh 6 60 660 OUT`
(`metal/cputime.txt`). Medians of 6 rounds, ms per frame:

| Scene | before | cpu | gpu |
|---|---|---|---|
| city-street | 3.47 | 3.05 | 3.19 |
| city-aerial | 3.12 | 3.10 | 3.62 |

What the GPU draws: the last timed frame's settled counts, the same in every
round.

| Scene | Path | Draws | Frustum-culled | Occluded | Triangles |
|---|---|---|---|---|---|
| city-street | before, cpu | 70 | 29,172 | - | 129,938 |
| city-street | gpu | 87 | 29,172 | 6,929 | 8,150 |
| city-aerial | before, cpu | 36 | 2,854 | - | 480,002 |
| city-aerial | gpu | 87 | 2,854 | 178 | 443,618 |

On the street, occlusion removes 94 % of the triangles left after frustum
culling. From above, little is hidden. The extra draws are one indirect draw
per batch per pass, most of which draw nothing.

What this says about CPU time: on Metal, the GPU path saves about a fifth of
`render`'s time on the street (gpu / before 0.81) and about nothing from above
(0.97). Against the same binary culling on the CPU, it saves 14 to 17 %. The
saving is modest because the frustum test was never the large part of
`render` on this tier. A `sample` profile of the GPU path shows the remaining
time in work both paths share:

- sorting the 40,000 instances into batches (`build_batches`)
- copying them into the instance upload
- computing their world bounds
- encoding the passes

The occlusion test then saves GPU time (8,150 triangles against 129,938),
which the CPU figures do not show.

Process CPU time does not separate the paths beyond the noise (3.0 to 3.6 ms
for every variant). The GPU path's counts readback and its compute passes add
driver work that offsets what the CPU test no longer does.

## Chrome (WebGPU and WebGL2)

    cd packages/engine
    bun tests/web/reference-gfx.ts --scene city-street,city-aerial --time 120 --perf-out ../../tests/evidence/P3/culling/chrome/rN
    bun tests/web/reference-gfx.ts --scene city-street,city-aerial --time 120 --perf-out ../../tests/evidence/P3/culling/chrome/rN --cpu-culling
    bun tests/web/reference-gfx.ts --scene city-street,city-aerial --time 120 --perf-out ../../tests/evidence/P3/culling/chrome/rN --webgl

These were run for r1, r2 and r3, interleaved. Each command first compares
the frame (PASS every time). It then makes 7 runs of 120 frames, each ending
in a capture whose readback waits for the GPU. `frame_ms` is the best run per
frame. `submit_median_ms` is the median wall time of one `og_gfx_render(1)`
call, which is the CPU cost of a frame. The files are
`chrome/rN/{webgpu,webgpu-cpu-culling,webgl2}/<scene>.json`, each with its
runs, the load average and the last frame's counts.

ms per frame, best run, over r1 / r2 / r3:

| Scene | WebGPU, GPU culling | WebGPU, CPU culling | WebGL2 (CPU culling) |
|---|---|---|---|
| city-street | 0.94 / 1.21 / 1.16 | 1.55 / 1.71 / 1.76 | 1.93 / 2.18 / 2.06 |
| city-aerial | 1.04 / 1.26 / 1.17 | 1.55 / 1.76 / 1.81 | 1.84 / 2.27 / 2.13 |

Median of one render call, ms, over r1 / r2 / r3:

| Scene | WebGPU, GPU culling | WebGPU, CPU culling | WebGL2 |
|---|---|---|---|
| city-street | 0.9 / 1.2 / 1.1 | 1.6 / 1.7 / 1.8 | 2.0 / 2.2 / 2.1 |
| city-aerial | 0.9 / 1.3 / 1.2 | 1.5 / 1.8 / 1.8 | 1.8 / 2.3 / 2.2 |

The browser's "before" is the CPU path. Before this round, WebGPU culled the
same way `--cpu-culling` culls now, and WebGL2 is unchanged.

On WebGPU the GPU path takes about 35 % less time per frame and per render
call, on both scenes. The wasm build pays more for the CPU-side frustum loop
than native code does. WebGL2 always culls on the CPU and is slowest of the
three.

The browser's GPU-path counts arrive late. The timing loop never yields to
the event loop, so the counts readback lands only at each run's capture, and
the last frame's `counts_lag` is 237 frames. That is why the JSON's GPU
`culled` (35,157 on the street) is not the CPU path's for the final camera
(36,889): the counts are from an earlier frame of the walk. The frame and its
comparison are unaffected. Natively, `finish_stats` waits and the counts are
exact.

## Same frame on both paths

`crates/og-tests/tests/culling.rs` renders eight scenes with
`Culling::Gpu` (the GPU path at any size) and with `Culling::Cpu` over
several frames each. The scenes are city-street, city-aerial,
gltf-reference, point-spot-lights, water-planar, msaa, shader-hooks and
showcase. The checks are:

- the same picture: no channel off by more than 2 outside 0.01 % of the
  pixels, RMSE at most 0.002 (the review tightened this from the gfx
  references' tolerance, which would hide a missing object)
- the GPU path on every frame exactly on the full tier, the CPU path on
  every frame with `Culling::Cpu`
- the same frustum-culled count
- never more triangles drawn on the GPU path
- the triangles the GPU draws plus those it hides (`occluded_triangles`,
  counted per hidden instance) no fewer than the CPU path's
  `in_view_triangles`
- on city-street, most of what is in view occluded

A second test checks that `Culling::Auto` culls city-street on the GPU and
the showcase on the CPU.

The original checks passed on Metal; the tightened ones were added after
this record and have not been run here. Broken once each:

- With the occlusion margin made permissive (`near > farthest - 0.002` in
  `cull.wgsl`), it failed: city-street RMSE 0.036, city-aerial 0.079,
  gltf-reference 0.053, with visible geometry hidden.
- With the bounding sphere's radius shrunk, it failed: the GPU's
  frustum-culled counts exceeded the CPU's (gltf-reference 6 against 5,
  shader-hooks 103 against 95).

Both were restored and pass again.

On lavapipe (`vulkan-cpu`, the Linux CI tier), every one of the 47 og-gfx
scenes with GPU culling compares with its reference exactly as the CPU path
does: the same RMSE and the same outliers. One fix came first. llvmpipe
answers level 0's size for every level of `textureDimensions(pyramid, level)`,
so a box at the right edge read past the level and was hidden. The level size
now comes from the screen's (53cdc86).

The gate's determinism step compares the math digest and the world hash
between the host and wasm32 (`tools/wasm-determinism/run.sh`). Rendering is
not part of either, and the step stays green. The full gate result is in
docs/phase-1.md's round section.

## Small scenes: the GPU path's per-frame overhead

The full gate on 6a99ead failed bun-test on 5 s timeouts in render.test.ts
(load 150 to 300). On the debug addon, the trails test takes 1.80 to 1.89 s
with GPU culling and 1.37 to 1.41 s with `Culling::Cpu` as the default. The
hit-flash test takes 1.07 to 1.32 s against 0.81 to 0.86 s. The cause is not
setup: `GpuCulling::new` takes 2 to 5 ms and the first pyramid 0.75 ms.

Release og-gfx, three interleaved rounds at load about 160 to 220:

    for s in showcase point-spot-lights; do for v in gpu cpu; do
      e=""; [ $v = cpu ] && e="OG_GFX_CULLING=cpu"
      env $e target/release/og-gfx $s --frames 200; done; done

| scene             | gpu frame ms        | cpu frame ms        |
|-------------------|---------------------|---------------------|
| showcase          | 6.12 / 3.53 / 3.75  | 3.15 / 3.17 / 3.15  |
| point-spot-lights | 1.22 / 1.64 / 1.04  | 0.58 / 0.60 / 0.62  |

The frames are small, so the fixed per-frame cost of the compute passes
(clears, pyramid, prefix sum, indirect args and the count readback) is
larger than the CPU culling it replaces.

What followed (branch `fix-culling`):

- `Culling::Auto` culls on the GPU only from `GPU_CULLING_MIN_INSTANCES`
  (16,384) instances. The city saves about 1.1 ms at 40,000 instances, the
  small scenes lose 0.4 to 1 ms, and the CPU test costs about 40 ns per
  instance, so the break-even lies between about 10,000 and 25,000.
  `Culling::Gpu` keeps the GPU path at any size.
- A frame with no occluder material, or no earlier camera visibility,
  skips the occluders' pass, their depth and the pyramid, and runs one
  compute pass.
- The count readback is copied only for frames after
  `Renderer::request_culling_counts`.

The threshold was not measured again after these cuts, which lower the
GPU path's fixed cost.
