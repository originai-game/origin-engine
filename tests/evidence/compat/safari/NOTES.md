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
| 26 black frames: every terrain-*, the terrain_scenes water-* (water-planar, -shadows, -underwater), grass-*, impostor-scatter, sprites-shadow-terrain | engine or WebKit bug, not legitimate | open. The common part is the terrain renderer, not GPU culling: the water scenes in fork_scenes.rs (no terrain) pass. The frames are black including the sky, which reads as a rejected command buffer (a WebGPU validation error drops the whole submit), not shading. The earlier run's log was empty because the browser prints uncaptured errors only to its own console: og-tests-web now fails the capture with them (60fe1ae67). Next: gfx `--scene terrain-steep,spheres-20k` in Safari. |
| spheres-20k torn triangles | engine or WebKit bug | open; with and without `--cpu-culling` next. |
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
