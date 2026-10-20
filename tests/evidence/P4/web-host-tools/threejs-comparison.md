# The design §7 comparison with three.js (web-host-tools round)

Design §7: "reference scene (500 instanced objects, 2 shadowed lights, 20
skinned characters, PBR, post): ≥ 1.5× the frame rate of the same scene in
Three.js on the same device". The spike (docs/spike-results.md §3) measured
parity on GPU-bound instanced content and 7.5× on per-object content, with
its renderer only. This is the first measurement of the engine itself (the
publish build, ECS, TS systems, extraction, og-render) against three.js.

## How

```sh
cd tools/bench && bun install --frozen-lockfile && cd ../..   # three 0.180.0 (pinned in tools/bench/bun.lock)
sh packages/engine/scripts/build.sh                           # dist/web-publish (the gate's wasm)
bun tools/bench/threejs-compare.ts --frames 600 --warm 60 --rounds 2   # the timed runs (about 40 min, most of it the engine's sphere field)
bun tools/bench/threejs-compare.ts --shots                    # the screenshots (frame-rate limit on), then sips -Z 640
```

`tools/bench/threejs-compare.ts` builds both engine scenes with
`og build --publish`'s web half (`webBuild`, the 3D publish variant plus the
lanes variant), serves them and the three.js pages cross-origin isolated,
and runs every mode in its own headless Chrome for Testing 150
(`--use-angle=metal --enable-unsafe-webgpu --disable-gpu-vsync
--disable-frame-rate-limit`, 1280×720 at DPR 1), the modes interleaved
round by round. Results: `threejs-comparison.json` (every run, every frame
time, the load average before it), screenshots `threejs-*.png` (640 px).

Scenes:

- **reference**: `crates/og-tests/reference-scene` (500 objects, boxes and
  spheres in four PBR materials, each turning on its own rate; 20 skinned
  walkers on four clips; the sun with cascades and a spot light, both
  shadowed; sky IBL; AO, bloom, ACES, TAA) against
  `tools/bench/three-reference.js`: the same grid, materials, walkers
  (`SkeletonUtils.clone` + one `AnimationMixer` each), a shadowed
  DirectionalLight (one 2048² PCF soft map over 60 m) and SpotLight, `Sky`
  through PMREM as background and IBL, and an `EffectComposer` with
  GTAOPass, UnrealBloomPass, SMAAPass and OutputPass (ACES). Two forms:
  four `InstancedMesh` (every matrix rewritten per frame) and one `Mesh`
  per object (500 draws).
- **spheres**: `tools/bench/spheres` (20,000 spheres of 16 × 12 segments,
  one geometry and material, a TS system writing every sphere's
  `og.Transform.position` each tick; one directional light, no shadows, no
  post, MSAA 4×) against `tools/bench/three-spheres.js` (the same field
  moved every frame: one `Mesh` per sphere, 20,001 draws; or one
  `InstancedMesh` with every matrix rewritten; WebGLRenderer, and
  WebGPURenderer for the instanced form).

**A frame time** is the wall time from one frame to the next when frames run
back to back, each advancing the simulation exactly 1/60 s and rendering
once. three.js: the interval between consecutive `requestAnimationFrame`
callbacks (no vsync, no frame-rate limit). The engine: the round trip of one
`frame` call from the page to the engine Worker (one tick and one render,
as `packages/engine/bench/lanes.ts` times it). The engine's own display
loop ticks at a fixed 60 Hz of wall time, so with frames uncapped it would
render many frames per tick and do less work per frame than three.js; the
frame call keeps one tick per frame on both sides. GPU work is submitted,
not waited for, on both sides; over 600 back-to-back frames the GPU queue
throttles submission, so **sustained** (the timed frames' wall time over
their count, and fps = 1000 / sustained) is bounded by whichever of CPU and
GPU is slower. 60 warm-up frames are not counted. Figures are the mean of
the two rounds.

## Results (Apple M4 Pro, Chrome for Testing 150, 2026-10-20)

| scene | mode | p50 ms | p95 ms | sustained ms | fps |
|---|---|---|---|---|---|
| reference | engine WebGPU | 1.535 | 4.805 | 2.021 | 495 |
| reference | engine WebGL2 | 2.715 | 5.793 | 3.046 | 328 |
| reference | three WebGL2, InstancedMesh | 3.763 | 7.047 | 3.923 | 255 |
| reference | three WebGL2, one Mesh each | 4.425 | 8.175 | 4.559 | 219 |
| spheres | engine WebGPU | 174.69 | 186.837 | 173.872 | 6 |
| spheres | engine WebGL2 | 175.258 | 188.183 | 173.948 | 6 |
| spheres | three WebGL2, InstancedMesh | 2.485 | 4.425 | 2.646 | 378 |
| spheres | three WebGL2, one Mesh each | 12.788 | 15.825 | 13.117 | 76 |
| spheres | three WebGPU, InstancedMesh | 1.533 | 3.013 | 1.65 | 606 |

Each round's sustained figure was within 10 % of the other's for every
mode (`threejs-comparison.json` `runs`). Load averages (1/5/15 min) before
the runs: 41 to 48 in round 1, 72 to 120 in round 2 (other agents' builds
and a lavapipe container on the same machine): read the ratios, not the
absolute figures.

**Against design §7 (≥ 1.5× three.js's frame rate):**

- **Reference scene: met on WebGPU, not on the WebGL2 fallback.** Engine
  WebGPU against three.js WebGL2: 1.94× (InstancedMesh, 495 against 255
  fps) and 2.26× (one Mesh each). Engine WebGL2 against the same: 1.29×
  (InstancedMesh) and 1.50× (one Mesh each). The engine is still ahead on
  WebGL2 where the spike was 30–45 % behind its own WebGPU.
- **Per-object sphere field: failed by a wide margin.** The engine takes
  174 ms a frame (6 fps) on both tiers, against three.js's 13.1 ms with
  20,001 draws (13×, the wrong way) and 2.6 ms instanced (66×). The spike's
  renderer drew the same field in 4.6 ms. The cost is not the tick: the
  probe's `state` timings put `step` at 0.3 to 0.5 ms and `render`
  (extraction plus og-render's frame) at 165 to 200 ms. It is not GPU or
  tier bound either (WebGPU and WebGL2 are the same). Scaling the field
  (WebGPU, `render` medians of 20 frames, load 40 to 50):

  | spheres | frame ms | step ms | render ms |
  |---|---|---|---|
  | 2,000 | 5.20 | 0.24 | 4.04 |
  | 3,000 | 13.95 | 0.23 | 12.81 |
  | 4,000 | 17.82 | 0.24 | 16.88 |
  | 4,100 | 34.93 | 0.24 | 34.18 |
  | 4,500 | 38.97 | 0.25 | 37.81 |
  | 5,000 | 43.80 | 0.28 | 42.88 |
  | 10,000 | 87.97 | 0.27 | 87.31 |
  | 20,000 | 174.94 | 0.34 | 173.93 |

  The render cost doubles between 4,000 and 4,100 instances of one mesh
  and grows at about 8.7 µs per instance beyond, and from 2,000 to 4,000 it
  grows faster than linearly: something per instance in extraction or in
  og-render's instance path (a step near 4,096 instances, then per-instance
  work) is far off the spike's. That is an engine defect to find (not
  investigated here); the reference scene, with 500 objects, stays below
  it.

## What is not the same on both sides

- Antialiasing: the engine's reference scene uses TAA; three.js uses SMAA
  (three's TAARenderPass re-renders the whole scene per sample on a moving
  scene, which would not be the same work). SMAA is the cheaper of the two,
  so this favours three.js.
- Shadows: the engine's sun has cascades; three's one 2048² map over 60 m.
- Light, exposure and sky are not matched in value (three's frame is
  brighter and paler; `threejs-reference-*.png`): the geometry, draw
  structure, shadowed lights, skinning and passes are what is matched.
- The engine renders in a Worker on an OffscreenCanvas and leaves the page
  thread free; three.js renders on the page's thread. The engine's frame
  time includes a postMessage round trip (tens of microseconds).
- The engine page offers the lanes variant (isolated, 8 Workers), as the
  product page does on an isolated host; the reference scene's lanes do
  not split (docs/web-export.md §Lanes), so this changes nothing there.
- With the frame-rate limit off three's rAF intervals are bimodal (p50
  well under sustained in some smoke runs); sustained is the fairer figure
  and is what the ratios above use.
- Screenshots need the frame-rate limit on (with it off
  `Page.captureScreenshot` never lands in headless Chrome), so they come
  from a separate untimed pass (`--shots`).
