# Forward+ clustered lighting (owed row 30)

Apple M4 Pro, macOS, 1280×720, with other agents' builds on the machine
(load average 40–63 during the native runs), so every number is the best of
its runs and carries ±15 % noise.

## What changed

The 8-light uniform loop (`frame.lights[i]` for every fragment, lights past
the eighth dropped) is replaced by a CPU cluster build (`og-render`
`src/clusters.rs`): 16 × 9 tiles × 24 logarithmic depth slices, lights
culled to the view, up to 256 drawn, at most 64 per cluster, the tables in
storage buffers (full tier) or 16 KiB uniform buffers (WebGL2). README
§Forward+ has the design and its limits.

## Frames

- `lights-256-before-8-light-loop.png`: og-tests' `lights-256` scene on the
  old renderer: eight lights drawn, the far strip lit, the rest dark.
- `lights-256-after-clustered.png`: the same scene clustered: 243 lights in
  view drawn, 13 culled (the reference `crates/og-tests/references/metal/lights-256.png`).
- Brute force against clusters: with the cluster box forced to the whole
  grid and the cap raised to 255 (every pixel shading all 243 lights, the
  old loop's shape at this light count), the frame is identical to the
  clustered one (RMSE 0.00000, max channel difference 0), at 5.76 ms per
  frame against 0.557 ms.
- Every existing reference (og-tests on metal, the 13 fx-showcase stations)
  passes at RMSE 0 or as before: `point-spot-lights` and `showcase` are byte
  identical.
- `fx-showcase-lights-tiers.png`: the fx-showcase `lights` station (256
  `og.PointLight` / `og.SpotLight` entities): top left `og run` before (eight
  lights), top right metal after, bottom webgpu and webgl2 after (Chrome for
  Testing 150). Against metal: WebGPU 0.000 % outliers, RMSE 0.00101; WebGL2
  (the uniform tables) 0.000 %, RMSE 0.00045.

## Frame time

Native: `cargo test --release -p og-tests --test perf -- --ignored`, best of
7 runs of 100 frames (CPU and GPU: each run ends with `wait_idle`), three
invocations each.

| Scene (metal) | Lights drawn | 8-light loop | Clustered |
|---|---|---|---|
| `point-spot-lights` (7 lights, 2 shadowed) | 7 | 0.420, 0.465, 0.661 ms | 0.455, 0.530, 0.558 ms (baseline recorded 0.452) |
| `lights-256` | 8 before, 243 after | 0.242, 0.256, 0.244 ms | 0.574, 0.579, 0.578 ms (baseline recorded 0.546) |
| `showcase` (4 lights, TAA, effects) | 4 | 2.762, 4.350, 3.477 ms | 3.144, 3.035, 3.368 ms |
| `lights-256` brute force (every pixel, all 243) | 243 | | 5.76–6.99 ms |

The CPU side (og-gfx's `cpu=`, 300 frames): `lights-256` 0.261 ms before
(eight lights, no build) and 0.272–0.298 ms after, so the cluster build and
its three uploads cost 10–40 µs for 256 lights; `point-spot-lights` 0.42–0.63
before and after alike. A compute pass would not beat that: one dispatch and
its barrier cost about as much on this machine, and WebGL2 needs the CPU
build anyway.

Browser: `bun tests/web/reference.ts [--webgl] --script 0.5:station:<s>
--shots <s>@90 --time 600` (new: 600 frames of one tick and one render
each after the shot, ended by a capture so the GPU's work is counted; the
milliseconds per frame are the whole run over 600), Chrome for Testing 150
headless, the dev export, three runs each.

| Station | Tier | 8-light loop (ms per frame) | Clustered (ms per frame) |
|---|---|---|---|
| `lights` (256 lights; 8 drawn before) | WebGPU | 1.019, 1.174, 1.175 | 1.690, 1.999, 1.969 |
| `lights` | WebGL2 | 1.815, 1.895, 1.922 | 2.237, 2.292, 2.363 |
| `fx` (one shadowed point light, particles) | WebGPU | 1.201, 1.310, 1.337 | 1.352, 1.435, 1.326 |
| `fx` | WebGL2 | 2.044, 1.988, 2.044 | 2.041, 1.945, 1.908 |

So a station with one light costs what it did, and 256 lights cost about
0.8 ms more than 8 on WebGPU and 0.4 ms on WebGL2 at 1280×720, far inside
a 16.7 ms frame.

## Tests (`crates/og-tests/tests/lights.rs`)

- `lights_256_matches_its_reference`: the frame, every light drawn or
  culled, nothing over a cap.
- `moving_the_depth_slices_changes_nothing`: the near and far planes moved
  three ways (perspective) and once (orthographic): every light lands in
  other slices, at most 3 pixels change (where geometry meets the ground,
  the depth buffer's precision).
- `tile_boundaries_leave_no_step`: on a plain lit ground the light changes
  across each tile boundary as little as beside it.
- `past_the_cluster_cap_the_brightest_lights_stay`: 100 lights at one place
  shade as the 64 brightest alone (max difference ≤ 1).
- `past_max_lights_the_faint_far_ones_are_dropped`: 256 bright and 40 faint
  far lights draw as the 256 alone; stats (256, 40).
- `lights_outside_the_view_are_culled`: ten bright spots behind the camera
  change no pixel and are counted.
- `gfx.rs` `lights_beyond_the_limit_are_counted` now asserts the 256 limit.

Each was broken once (`break-and-restore.md`).
