# Safari compatibility: first pass (WIP, paused 2026-11-05)

macrunner (Apple M4 Pro), base origin/main 38b9b7c3a. Paused by the owner so
feature work comes first. Nothing here is fixed yet: these are the first
runs and the open questions.

## What is available

- macOS 27.2 (26B5091g), Safari 27.2 (22625.2.5.11.1), Xcode 27.2 (27B52),
  one Simulator runtime: iOS 27.0 (24A434). Xcode 27 ships no Simulator.app
  (DeviceHub.app instead).
- `sudo -n safaridriver --enable` works here (passwordless sudo). The first
  WebDriver session timed out until Safari had been opened once by hand
  (`open -a Safari`); after that, sessions open in about 1 s. A stale
  `safaridriver -p 4444` from 2026-11-02 (pid 61892, not this stream's) is
  still listening; runs use their own port.
- macOS Safari has WebGPU with a working adapter: `adapter-macos.json`
  (info, features, limits, WGSL language features); Chrome for Testing 150
  on the same Mac for comparison: `adapter-chrome.json`.
  - Safari lacks `dual-source-blending`, `subgroups` and
    `texture-component-swizzle` (Chrome has them); Safari alone has
    `float16-renderable` and `float32-renderable`. `info` is all "apple",
    no subgroup sizes.
  - Limits differ from Chrome's: maxColorAttachmentBytesPerSample 64 (Chrome
    128), maxBufferSize and maxStorageBufferBindingSize 2 GiB (4 GiB),
    maxComputeWorkgroupSizeZ 1024 (64), min{Uniform,Storage}BufferOffsetAlignment
    32 (256), maxBindGroups 11 (4), storage buffers/textures per stage 44
    (10/8), maxUniformBufferBindingSize 2 GiB (64 KiB), no maxImmediateSize.
    The engine asks for downlevel defaults (og-render init.rs), all within
    both.
- iOS Simulator (iPhone 18 Pro, iOS 27.0): Safari has `navigator.gpu` but
  `requestAdapter()` answers null (page served from 127.0.0.1, a secure
  context; UA "iPhone OS 18_7 ... Version/27.0", DPR 3, 402x714). So the
  Simulator is for layout and behaviour only: the shell shows its
  no-WebGPU card there.
  - safaridriver did not attach to the Simulator ("Could not find any
    session hosts that match the requested capabilities") even with Remote
    Automation written to the Simulator's prefs; `xcrun simctl io
    screenshot` hung. The route chosen instead: an XCUITest that drives
    Safari (tests/web/ios/, tests/web/ios-sim.ts). Its first run failed only
    because the harness ran xcodebuild with spawnSync, which blocked its own
    control server; fixed (Bun.spawn), not yet rerun.

## Harness added

- `packages/engine/tests/web/webdriver.ts`: Safari over classic WebDriver,
  the part of cdp.ts's Browser/Page the runners use (viewport, navigate,
  reload, close, eval in the page or a frame, pointer and key input,
  screenshot); console errors and uncaught errors are collected in the page
  and returned with each eval.
- `--safari` on reference.ts, reference-gfx.ts, reference-ui.ts (and
  reference-e5.ts, which forwards its arguments) and wire.ts. Safari's
  frames are their own tier, `webgpu-safari`, judged against that tier's
  reference, else `webgpu`, else `metal`. No webgpu-safari reference is
  recorded.
- `og run --device safari` (src/node/device.ts): this Mac's Safari through
  the same WebDriver driver the iPhone target uses, at localhost. Not run
  yet.

## First triage (Safari only; no Chrome control run yet)

The Chrome control runs (same tests in Chrome for Testing) were queued but
stopped by the pause, so "Safari-specific" below is a hypothesis wherever it
says so.

| Run | Result | First reading |
|---|---|---|
| fx-showcase stations (reference.ts) | 4 of 5 | lens RMSE 0.01088 (limit 0.01): the animated figure behind the sphere is posed differently (frames/lens-*.png). Likely the same as the stylise skinning row below. |
| fx-showcase boot | 61 pipelines made behind the gate in 23,045 ms | about 380 ms a pipeline in Safari; Chrome's number not measured yet. For the web-boot stream if Chrome is far lower. |
| og-tests gfx (reference-gfx.ts) | 170 of 206 | see below |
| gfx: GPU-driven paths | 26 scenes at 51-100 % outliers | spheres-20k (20,000 instances: GPU culling at >= 16,384) draws torn triangles; every terrain-*, water-*, grass-*, impostor-scatter and sprites-shadow-terrain frame is black (frames/gfx-big-fails.png, terrain-steep.png). Hypothesis: the GPU-culling / indirect-draw path (cull_gpu.rs, cull.wgsl), which these share. Next step: `--cpu-culling` on spheres-20k, and the error-scope probe (WebGPU validation and WGSL messages) that was queued. Probably an engine bug or a WebKit bug; not a legitimate difference. |
| gfx: stylise-* (8), hooks-screen-ao, ui-layers | 0.8-6.6 % outliers | stylise-pixel shows the skinned character with its limbs scattered (frames/stylise-pixel.png): skinning differs in Safari for that model, while skinning and skinned-hooks pass. |
| gfx: point-spot-lights, post-sharpen | RMSE 0.015-0.016 | small; needs the Chrome control. |
| og-ui kit (reference-ui.ts) | style packs: cartoon, pixel 4 of 4 sheets; fantasy-phone, scifi-phone 27 of 29; fantasy, scifi errored; frames-desktop 2/28, frames-phone 1/28, players 0/1, locales 4/10 | The frames differ in content, not rendering (frames/raceHud.png: another HUD layout and track name): the kit changed after its metal references (002f69b06, 2026-11-04 04:05, vs ui-kit commits that day) and those references are being re-recorded on main. Not Safari. |
| og-probe wire (wire.ts) | 110 of 117 | (1) dev isolated: the first tap on the gate did not start the clock; later runs' taps did. Likely the first pointer action in a new Safari window (harness), to confirm. (2) record {seconds: 1} answered 0 bytes in 2 of 4 runs (video/webm; codecs=vp8): shell/wire.ts stops a start(1000) MediaRecorder at 1000 ms; an engine race to fix (a probe page, .amp-side, was queued). (3) after Page.reload the parent page's `window.wire` never appeared in 4 runs: the harness's reload over WebDriver, to debug. |
| fx-showcase E5 and looks, iOS Simulator | not run | stopped by the pause |

## Resumed (2026-11-05): triage

Rebased onto origin/main 6f27e2201 in lane-12. Development has the GPU
until the wave-end run reports, so only the GPU-free rows moved so far;
`run.sh` (beside this file) is how every remaining run is made, one GPU
slot each, queued behind development's waiters.

| Finding | Class | State |
|---|---|---|
| `record {seconds: 1}` answered 0 bytes (2 of 4 runs) | engine bug (Safari behaviour the engine tripped) | fixed, f5958a92b. Safari 27.2 answers a `stop()` on a MediaRecorder timeslice boundary with one empty chunk: on a 2D canvas's capture stream, `start(1000)` stopped at 1000 ms was empty 16 of 16, `start()` 0 of 16. The wire now records without a timeslice and answers `failed` on no data. wire.ts `--safari` is the test that fails on the old code; to rerun. |
| `window.wire` never appeared after a reload (4 of 4) | harness | fixed, 60fe1ae67: webdriver.ts's eval JSON-stringified a function to `undefined`, so `until("window.wire")` never passed (CDP returns `{}`). A static page's reload now passes 3 of 3. |
| first tap on the gate did not start the clock (1 run) | harness (probably) | 60fe1ae67 polls for 3 s. Taps reach a cross-origin frame on the first try in a plain page; the transcript shows the `tick` after the tap answered 594 ms late and stale, the clock running by the next calls. Hypothesis to confirm on a GPU run: the gate's `fullscreen()` transition stalls Safari's page. |
| 26 black frames: every terrain-*, the terrain_scenes water-* (water-planar, -shadows, -underwater), grass-*, impostor-scatter, sprites-shadow-terrain | engine bug, fixed on main | terrain-steep now passes in Safari against metal (0.000 %, RMSE 0.00007; runs/safari-gfx). The cause was main's b78de9cf8: `terrain_lit.wgsl` took derivatives after a non-uniform return, the browsers refuse the `terrain` module, so every frame's command buffer was invalid. The rest of the cluster is to confirm on the full gfx tier. |
| spheres-20k torn triangles | engine or WebKit bug, GPU-driven path | open; GPU runs paused by the owner. With `--cpu-culling` it passes (0.093 %, RMSE 0.006, 751 ms); with GPU culling (>= 16,384 instances) it draws a few huge flat triangles in the wrong colours (99.5 % outliers), takes 33.8 s and raises no uncaptured WebGPU error. Ruled out by reading: `indirect-first-instance` (Safari's adapter has it, init.rs requests it, and cull.wgsl writes first_instance 0 and base_vertex 0 anyway: each batch's instances come from a vertex-buffer slice at `region(g, first)`), the DX12 base_vertex fix (DX12 only), and the storage offset alignment (no offset storage bindings: instance slices are vertex buffers at 64-byte steps, indirect offsets 20-byte steps). The picture reads as corrupted indices or vertices, not misplaced instances, and the CPU path differs only in `drawIndexed` vs `drawIndexedIndirect` over the same buffers. Two hypotheses were tested with minimal repros (probes/, one slot each, Safari 27.2) and both are ruled out: probes/indirect-draw.ts, `drawIndexedIndirect` with compute-written args against `drawIndexed`, instance slices at 0, 20,000 and 40,000 instances in: 0 differing pixels, no errors, 3-7 ms; probes/pass-order.ts, cull.wgsl's mark, block-sum, scan and prefix chain in one compute pass against one pass a dispatch, 20,000 and 100,000 elements, 20 runs each: 0 mismatches either way. (Its first run was void: the probe's own WGSL mixed `*` and `>>` without parentheses.) The real cull is not the cause either: `reference-gfx.ts --cull-dump` (a dev-only readback of the cull's buffers after the frame) wrote byte-identical dumps in Safari and Chrome for spheres-20k (scan's 131,072 words, the 64 words of draw args, the 8 MB of instance regions, the rest), while Safari drew the frame torn and Chrome right (0.093 %). So the first stage that differs is the draw that consumes those args: indexed-indirect draws of 1,152 indices x 19,028 and 16,717 instances (runs/cull/spheres-20k-args.json), at a scale probes/indirect-draw.ts (6 indices x 16 instances) did not reach. The vertex-count hypothesis (the products are past 2^24) is ruled out too: probes/draw-size.ts swept 1,152 indices x 14,000, 14,563, 14,564, 15,000, 16,717 and 19,028 instances at a 64-byte stride, and 96, 6 and 12,288 indices at products just under and over 2^24, direct and indirect with compute-written args: every cell right, no errors (draw-size.out.txt). The faithful rows (the scene's args offsets and instance regions, instances written by a compute pass in the same or a separate command buffer, busy compute) and probes/two-draws.ts (two meshes' indirect draws in one pass, args five words apart, 1,152 indices x 20,000) all draw right. Engine-side bisect (reference-gfx.ts --bisect, og-tests-web og_gfx_bisect; GPU culling against CPU culling in the same Safari): meshlets off, the cull submitted apart, the same material for all, the same mesh for all: still torn. Only the spheres' batch (`first`): right at 1,000 and at 20,000. 100 spheres + ground, shadows off: right; 1,000 + ground, shadows off: torn; 1,000 + ground with shadows: right; 2,000 + ground with shadows: torn. Torn frames take 13-31 s on the GPU against under 1 s, which reads as a GPU fault or timeout, not a wrong draw. So it needs a second batch and enough instances, with the threshold moving with the number of passes; next: the prepass (four vertex buffers, previous transforms) against the lit pass, one at a time. |
| skinned limbs scattered (stylise-*, fx lens) | engine or WebKit bug | open. |
| point-spot-lights, post-sharpen RMSE 0.015-0.016 | unknown | needs the Chrome control. |
| 61 pipelines in 23 s behind the gate | performance, shared with web boot (T-01a1a707) | needs Chrome's number on the same Mac. |
| og-ui kit frames | stale Metal references then | rerun against current main. |
| iOS Simulator: `requestAdapter()` null | legitimate (no WebGPU in the Simulator) | noted, not chased. |

## To resume

Run the Chrome controls first (fx, gfx, e5, ui, wire) and compare each
Safari frame with Chrome's as well as metal's, then take the GPU-driven
cluster (spheres-20k with and without --cpu-culling, with WebGPU error
scopes), the skinning difference, the MediaRecorder race and the pipeline
build time. Every Safari run takes the machine lock (scripts/lock.sh) under
a holder whose command line contains `check.sh`, or other waiters take the
lock over as stale (og_lock_alive).
