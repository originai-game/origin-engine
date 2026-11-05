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
| spheres-20k torn triangles | WebKit bug, engine workaround | fixed, on main at 498258db0 and a24667014 (the bisect is kept as it was found; its cause ends the row). WebKit's Metal backend races indirect draws that share an args buffer (bug 321876, fixed upstream in 6dca6330db, not in Safari 27.2); on WebKit every GPU-culled draw now reads args of its own (`RenderInfo::indirect_per_draw`, indirect_own.rs). Safari's spheres-20k passes against metal (0.124 %, RMSE 0.00757) in 143 ms, against 31 s torn. Before the fix: With `--cpu-culling` it passes (0.093 %, RMSE 0.006, 751 ms); with GPU culling (>= 16,384 instances) it draws a few huge flat triangles in the wrong colours (99.5 % outliers), takes 33.8 s and raises no uncaptured WebGPU error. Ruled out by reading: `indirect-first-instance` (Safari's adapter has it, init.rs requests it, and cull.wgsl writes first_instance 0 and base_vertex 0 anyway: each batch's instances come from a vertex-buffer slice at `region(g, first)`), the DX12 base_vertex fix (DX12 only), and the storage offset alignment (no offset storage bindings: instance slices are vertex buffers at 64-byte steps, indirect offsets 20-byte steps). The picture reads as corrupted indices or vertices, not misplaced instances, and the CPU path differs only in `drawIndexed` vs `drawIndexedIndirect` over the same buffers. Two hypotheses were tested with minimal repros (probes/, one slot each, Safari 27.2) and both are ruled out: probes/indirect-draw.ts, `drawIndexedIndirect` with compute-written args against `drawIndexed`, instance slices at 0, 20,000 and 40,000 instances in: 0 differing pixels, no errors, 3-7 ms; probes/pass-order.ts, cull.wgsl's mark, block-sum, scan and prefix chain in one compute pass against one pass a dispatch, 20,000 and 100,000 elements, 20 runs each: 0 mismatches either way. (Its first run was void: the probe's own WGSL mixed `*` and `>>` without parentheses.) The real cull is not the cause either: `reference-gfx.ts --cull-dump` (a dev-only readback of the cull's buffers after the frame) wrote byte-identical dumps in Safari and Chrome for spheres-20k (scan's 131,072 words, the 64 words of draw args, the 8 MB of instance regions, the rest), while Safari drew the frame torn and Chrome right (0.093 %). So the first stage that differs is the draw that consumes those args: indexed-indirect draws of 1,152 indices x 19,028 and 16,717 instances (runs/cull/spheres-20k-args.json), at a scale probes/indirect-draw.ts (6 indices x 16 instances) did not reach. The vertex-count hypothesis (the products are past 2^24) is ruled out too: probes/draw-size.ts swept 1,152 indices x 14,000, 14,563, 14,564, 15,000, 16,717 and 19,028 instances at a 64-byte stride, and 96, 6 and 12,288 indices at products just under and over 2^24, direct and indirect with compute-written args: every cell right, no errors (draw-size.out.txt). The faithful rows (the scene's args offsets and instance regions, instances written by a compute pass in the same or a separate command buffer, busy compute) and probes/two-draws.ts (two meshes' indirect draws in one pass, args five words apart, 1,152 indices x 20,000) all draw right. Engine-side bisect (reference-gfx.ts --bisect, og-tests-web og_gfx_bisect; GPU culling against CPU culling in the same Safari): meshlets off, the cull submitted apart, the same material for all, the same mesh for all: still torn. Only the spheres' batch (`first`): right at 1,000 and at 20,000. 100 spheres + ground, shadows off: right; 1,000 + ground, shadows off: torn; 1,000 + ground with shadows: right; 2,000 + ground with shadows: torn. Torn frames take 13-31 s on the GPU against under 1 s, which reads as a GPU fault or timeout, not a wrong draw. So it needs a second batch and enough instances, with the threshold moving with the number of passes; The threshold is not a threshold: the same row (`gpu,noshadow,keep:518`, 518 spheres + ground, passes `main`, `tonemap`) drew right in some pages and torn in others (bisect-19..23), and once a page tears every later GPU-culled renderer in it tears too, while 516 and the 16x12 mesh re-added (`sphere:16x12`) passed in a page that had not torn. In torn frames the readback (bisect `dump`) shows the args right (`[1152,518,0,0,0, 6,1,0,0,0]`, instance count 519) and the first pass's culled instance region identical to the uploaded records. The new `direct` knob (dev-only, `Renderer::debug_cull_direct`: the lit pass draws the same culled regions with `drawIndexed` instead of the args) drew right in every row, including the one where its indirect twin tore in the same page (bisect-23: indirect torn 16.3 s, direct right 111 ms). So the fault is in Safari's `drawIndexedIndirect` itself, fed correct args and buffers. **Cause (WebKit's source):** WebKit's Metal backend (the safari-7625.2.x branches; checked at safari-7625.2.8.18, d953bbedb) clamps every indirect draw whose pipeline has vertex buffers into one 20-byte slot owned by the INDIRECT `GPUBuffer` (Buffer.mm: `m_indirectIndexedBuffer`; RenderPassEncoder.mm `clampIndirectIndexBufferToValidValues`, two point-draw clamp passes, then `drawIndexedPrimitives:... indirectBuffer:` at that slot, offset 0), and nothing orders draw N's argument fetch before draw N+1's clamp writes the same slot (on Apple8+ `memoryBarrierLimit` is UINT32_MAX, so the pass is never split). Draw N then reads draw N+1's counts or a mix: the 6-index plane drawn with the spheres' 1,152 x 518, torn triangles and a tiler timeout (16-31 s frames). Timing-dependent, which is why the probes passed and the threshold moved. Fixed upstream by WebKit 6dca6330db (2026-08-17, bug 321876, rdar://185057290: per-draw scratch), not yet in Safari 27.2. **Engine workaround (keeps GPU culling):** on WebKit (`navigator.userAgent`; `RenderInfo::indirect_per_draw`, forced by `RenderDesc::indirect_per_draw`) cull_gpu.rs copies every (pass, batch) draw's five words to a 20-byte INDIRECT buffer of its own after the cull (the occluders' group before the occluder depth pass), and every GPU-culled draw (lit, prepass, shadows, skinned, occluders) reads `GpuCulling::draw_args`: a slot per draw. Safari after it: `gpu,noshadow,keep:518` x4 and the whole scene x2 all equal to CPU culling (0.000 %), the whole scene 143 ms against 31 s. Native test: og-tests culling.rs `per_draw_indirect_args_draw_the_shared_buffers_frame` (per-draw args draw the shared buffer's frame exactly). Not yet covered by the workaround: the other indirect users that share one args buffer across draws in a pass (crowd_gpu.rs, scatter.rs's cull, grass.rs LODs, the meshlet path's `draw_indirect` where its pipeline has vertex buffers); their Safari frames passed, but they are exposed to the same race. |
| skinned limbs scattered (stylise-*, fx lens) | not reproduced on current main | every stylise-* but stylise-print passes in Safari's gfx tier, and stylise-print fails identically in Chrome (below). Plausibly the same indirect race (skinned batches draw through the GPU cull's args). fx lens: the fx tier was stopped by a pause, to rerun. |
| point-spot-lights, post-sharpen RMSE 0.015-0.016 | not reproduced | both pass in Safari's gfx tier on current main. |
| 61 pipelines in 23 s behind the gate | performance, shared with web boot (T-01a1a707) | needs Chrome's number on the same Mac. |
| og-ui kit frames | stale Metal references then; now not Safari | rerun on current main: everything passes in Safari except kit-style-cartoon(-phone) raceHud cell 10 (tick 816; 0.536 % / RMSE 0.01997 and 0.174 % / RMSE 0.02315), which Chrome fails with the same numbers. For main or the art runner: a reference or browser-wide difference, not Safari. |
| iOS Simulator: `requestAdapter()` null | legitimate (no WebGPU in the Simulator) | noted, not chased. |

## Tiers on main (2026-11-05 evening), Chrome controls on the same build

Rebased onto origin/main af9bed852, then 51 commits later for the push.
One GPU slot a run (`run.sh`, driven by a scratch loop), Safari then
Chrome per tier. The ui tier's Safari run used the build before the
second rebase, its Chrome run the one after (og-ui unchanged between).

| Tier | Safari 27.2 | Chrome 150 (control) | Class |
|---|---|---|---|
| gfx (219 scenes) | 217 before the fix; spheres-20k passes after it | 217 | spheres-20k: WebKit bug, fixed in the engine (above). stylise-print (5.739 %, RMSE 0.05095) and hemisphere-light (RMSE 0.01076) fail identically in both: not Safari (a metal reference or a browser-wide difference, for main). Safari's tier took 452 s against Chrome's 229 s, 31 s of it spheres-20k's torn frame. |
| ui (kit frames, 14 sheets and states) | all but cartoon raceHud | all but cartoon raceHud, same numbers | not Safari (above). |
| fx (5 stations) | 5 of 5 | 5 of 5 | lens now passes (the scattered limbs are gone on main). Boot under main's `pipelines auto`: first frame at 495 ms (Chrome 431 ms), no warm-up behind the gate; the old 61 pipelines in 23 s is gone. The run is 17.3 s against Chrome's 3.2 s (it was 54 s): pipelines still made synchronously as frames need them, for web boot (T-01a1a707). |
| e5 (26 stations) | 22 of 26 | 22 of 26, the same four | environment, materials, volumetric, water fail with the same numbers in both (water 7.660 % / RMSE 0.04947): not Safari, for main. |
| looks (37) | 26 of 37 | 29 of 37 | 8 fail in both with the same numbers (rubber-hose, pencil-hatching, gothic-contour, decorative-storybook, detailed-doodle, black-figure-pottery, heavy-ink-shadows, anime-cel-3d; Chrome also paper-craft 0.613 %): for main. **Safari only, open:** bold-comic-illustration 1.417 %, illuminated-manuscript 2.038 %, voxel-worlds 0.143 % / RMSE 0.01242, sculpted-clay RMSE 0.01098. Same scene and poses (inspected); the extra outliers lie on the outline post effect's edges plus a light speckle, all over the frame (Safari against Chrome 1.4-2.0 %), so a difference in the edge detection's inputs (depth or normals precision, derivatives). Engine or legitimate not yet decided; no per-backend reference recorded. |
| wire (397 checks) | 397 of 397 | 397 of 397 | `record` answers a recording in every run (the fix is on main), `window.wire` after reload passes, and the first tap starts the clock without the 3 s poll: the slowest non-record call is the first `tick` at 152 ms (was 594 ms late). Closed. |
| tp (3 shots) | 2 of 3 | 1 of 3 | title (2.913 %, RMSE 0.07088) fails identically in both; Chrome also fails beacon (0.663 %), Safari passes it: not Safari. |
| examples | brasswick 9 of 9; hollowbell 1 of 9; meridian-loop 0 of 10 | brasswick 9 of 9; hollowbell 1 of 9, the same numbers; meridian-loop 8 of 10 (jump, slide RMSE 0.012-0.013) | hollowbell: not Safari, for main. **meridian-loop: Safari only, open**, every station (1.3-4.1 % outliers, RMSE 0.025-0.045 in the race, 0.013-0.016 on the menus). Same content (inspected); the difference lies on the scene's geometry edges and outlines everywhere, the UI panels clean: the same signature as the four Safari-only looks. Both render with TAA, but the runner drives one tick and one render a call, so frame pacing is not it, and gfx's taa, gi-room-taa and every stylise-outline scene pass in Safari. Next: cut a game frame down (antialias off, then the look's post passes off) to the first change that makes Safari agree. (The first example runs failed before a frame: brasswick and meridian-loop generate their music after a clone, tools/make-assets.ts, which run.sh now runs; ember-vale does not exist, hollowbell ran instead.) |

Same cause, more draws: the crowds' culled draws (crowd_gpu.rs) and
scatter's (scatter_cull.rs) also share one args buffer each with vertex
buffers bound, so they are exposed to the same WebKit race though their
Safari frames passed; the same per-draw copies cover them (indirect_own.rs,
with crowd-lods and impostor-scatter in culling.rs's per-draw test).
Grass and the meshlet path draw without vertex buffers, which WebKit does
not clamp.

## Edge difference in game frames: where the bisect stands (2026-11-06 00:50, handed off)

Everything below is meridian-loop in Safari 27.2 (`run.sh fx --project ... --script ... --shots ...`, one slot each).

- **Not the content, and not Safari's rendering of one frame.** The title shot alone (`--script "3:race" --shots title@72`) passes in Safari 3 runs of 3 (0.075 %, against Chrome's 0.073 %). It also passes with the playtest's whole script and only the title shot, and with `1.3:race:go` or `120:language:de` alone. The full playtest (`--playtest tests/frames.og.ts`) fails the same shot at 1.813 %, twice, reproducibly. Chrome full against Chrome alone: 0.001 %.
- **The same script with all ten shots in `--script/--shots` mode** fails only jump and slide (0.202 % / 0.114 %, as Chrome does). The same ten shots through `--playtest` fail ghost, pause, results, board and settings-de too, with the numbers of the first full run. `--playtest` and `--script` build the same script and shots (checked with `--print`). What differs is that `--playtest` bundles the playtest first (`bundleGame` → `dist/test-reference/.meridian-loop.playtest.js`), and the shot list's order and timing.
- **Shape of the difference:** a sub-pixel shift (about 0.5 px) and different softness on every geometry edge; the UI is clean. That reads as a different render scale or TAA jitter or history, not a different shader result.
- **Leading hypothesis, unverified:** automatic quality (og-render auto_quality.rs, og-runtime display.rs `before_render`/`after_frame`) is on by default (`autoQuality: true`) on a presenting host, and the web harness's `frame` calls feed it intervals. Safari's slower frames (its fx run is 17 s against Chrome's 3 s) can step the render scale down, and TAA then resolves at another scale. A run's history (how long its frames took) would then decide the frame, which fits run-to-run and mode-to-mode differences. probes/diag-auto-quality.patch (not applied) adds `diagScale`/`diagStep` to the frame answer and has reference.ts print each scale change. Apply it, `run.sh build`, then `run.sh example meridian-loop`, and read the `DIAG scale changes` line.
- **If confirmed, the root fix** is that a harness-driven frame (the probe's `frame` with the clock held, `pipelines.drawnWhole`) must not feed or apply automatic quality, as native `og run` draws at scale 1. That probably means turning auto quality off for probe-driven frames in og-runtime, with a test in crates/og-runtime/tests/display.rs. The four Safari-only looks (bold-comic 1.417 %, illuminated-manuscript 2.038 %, voxel-worlds, sculpted-clay) have the same edge signature and should be rechecked after the fix.
- Variants under .amp/in/ml/ (base, nowire, aaoff, nobloom: copies of meridian-loop with one look or render change) were made for an antialias and post-pass cut-down. Only base and nowire ran before the title-alone result showed that the frame itself renders right.

## To resume

Run the Chrome controls first (fx, gfx, e5, ui, wire) and compare each
Safari frame with Chrome's as well as metal's, then take the GPU-driven
cluster (spheres-20k with and without --cpu-culling, with WebGPU error
scopes), the skinning difference, the MediaRecorder race and the pipeline
build time. Every Safari run takes the machine lock (scripts/lock.sh) under
a holder whose command line contains `check.sh`, or other waiters take the
lock over as stale (og_lock_alive).
