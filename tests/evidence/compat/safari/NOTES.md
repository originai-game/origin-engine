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

## To resume

Run the Chrome controls first (fx, gfx, e5, ui, wire) and compare each
Safari frame with Chrome's as well as metal's, then take the GPU-driven
cluster (spheres-20k with and without --cpu-culling, with WebGPU error
scopes), the skinning difference, the MediaRecorder race and the pipeline
build time. Every Safari run takes the machine lock (scripts/lock.sh) under
a holder whose command line contains `check.sh`, or other waiters take the
lock over as stale (og_lock_alive).
