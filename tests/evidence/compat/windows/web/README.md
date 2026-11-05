# Web on Windows: triage (studio-win, 2026-11-05)

RTX 5060, Windows 11 26300, origin/main 012a073a3 with this branch's shell and
test commits; wasm and package from `scripts/check.sh --only wasm-build
wasm-opt` and `OG_GATE=1 sh packages/engine/scripts/build.sh`. Every frame
was judged against `metal` (no tier has `webgpu` references) under og-tests'
tolerance.

## Chrome 156 (Dawn, D3D12)

**Chrome draws what native DX12 draws.** Every failing browser frame was
re-rendered natively on DX12 at the same commit (`og test` for the
templates and games, `og-gfx --capture` for og-tests' scenes): the outliers
agree to within 0.01 % on fx-showcase's stations, looks, ember-vale,
brasswick, meridian-loop and ui-kit's frames, and og-tests' scenes are
pixel-identical (RMSE 0) bar one, ui-layers (below).

| Group | Result | Cause |
|---|---|---|
| fx (5 stations) | lens, stylised fail (RMSE 0.0109, 0.0106) | stale metal frames: 65f2bb67a (skinned meshes at their bind pose) turned the unposed trickster on every backend; at the references' commit DX12 passes (lens 0.035 %, stylised 0.024 %). `lens-trickster-bindpose.png` |
| fx-e5 (26) | 10 fail: baked, moon, physical, shaders, sky, volumetric, islands, terrain, water, weather (0.017 %, RMSE 0.0123) | stale metal frames, all ten re-recorded on main since (fc9a08bb6): at each reference's own commit DX12 passes (baked 0.000 %, physical 0.023 %, shaders 0.001 %, sky 0.021 %, volumetric 0.007 %, islands 0.063 %, water passes at eec965067; weather was not rerun at its commit); moon and terrain's residue at theirs was the `fract(sin)` star and caustic hash 71e70a0a2 replaced afterwards |
| fx-looks (37) | 12 fail | 11 moved since 947331d9d recorded them; detailed-doodle (11 %) is a posterize tie: the back wall's blue sits on stylise.wgsl's `floor(e * 5 + 0.5)` step (255 on metal, 204 here), so the whole uniform wall flips a level. `doodle-posterize-tie.png` |
| ui (ui-kit, 235 frames) | 208 fail | stale metal frames: native DX12 fails the same frames by the same amounts (kit-phone-raceHud 72.42 % native, 72.45 % Chrome); the references predate 437ccc897 (racing round 2) and the kits' later changes; kit-style-fantasy and -scifi are 4480x1440 where their playtests make 33 shots (4480x1800) |
| a11y | 22 of 23 | the fixture had no assets/ (fixed here); the contrast check's probe capture answers null on Chrome 156 / D3D12 (26/26 on macrunner, Chrome 150 / Metal): open |
| gfx (206 scenes) | 189 pass, 17 fail | 14 stale metal frames (their references' commits pass on DX12, or on Vulkan where DX12 there still had the indirect-draw `base_vertex` bug 0d77304b9 fixed: stylise-palette 0.26 %, stylise-print 0.009 % on Vulkan, `stylise-palette-at-ref.png`); sprites-array and tilemap-lit match `references/dx12` exactly (Dawn's D3D12 has the same nearest-texel ties); ui-layers: the custom UI material is not drawn (below) |
| ember-vale (3) | 3 fail (beacon 12.3 %) | same as native DX12 (12.27 %): stale |
| brasswick (9), meridian-loop (10) | 6 and 10 fail | same as native DX12 to within 0.6 points per frame: stale |
| lane-panic | 2 fail, "the engine Worker never attached" | a test bug on every OS: the Worker is `game.<hash>.js` since 63c5fcae5 and the test matched `game.js` (fixed by the web boot thread) |

**ui-layers' custom material (web only).** Cell (0,2), `og_render::
example_ui_material`, draws as the plain card in Chrome
(`ui-layers-custom-material.png`): ui/pass.rs held a custom UI material
back while its error scope was pending, and on the published wgpu
`popErrorScope` resolves on a later JS turn, after og-tests-web's frames
(one turn) are drawn. Being fixed in og-render by the web boot thread.

## Edge (Chromium, Dawn, D3D12)

Every group matches Chrome line for line: the same stations, frames and
scenes pass and fail, with the same outlier and RMSE figures (fx 5, fx-e5
26, looks 37, ui-kit 235, gfx 206, ember-vale 3, brasswick 9,
meridian-loop 10), a11y 22 of 23 on the same null capture, and
lane-panic's same harness failure. Nothing here is specific to Edge.

## Firefox 159 (wgpu in Gecko, D3D12)

Driven over WebDriver BiDi (`tests/web/bidi.ts`, `--firefox [<path>]`),
in a window (bidi.ts: WebGPU needs one), at origin/main cb2cd9216. Every
frame was judged against `metal` and against Chrome: ui-kit and the games
against a Chrome run of the same build, the fx groups against Chrome's run
at 012a073a3 above, whose figures they equal.

| Group | Firefox | Against Chrome |
|---|---|---|
| fx (5 stations) | 3 ok, lens and stylised fail | the same stations and figures (RMSE within 0.00001): the bind-pose stale frames above |
| fx-e5 (26 stations) | 16 ok, 10 fail | the same stations as Chrome |
| fx-looks (37) | 25 ok, 12 fail | the same presets as Chrome |
| ui (ui-kit, 233 frames, 2 packs) | 143 ok, 90 fail, 2 packs not judged | the same frames pass and fail; on all 233 the outlier figures are equal and RMSE differs by at most 0.00001; style-fantasy and -scifi fail on every browser where reference-ui.ts checks the 6 state shots against the screens' sheet (4480x1440, not 4480x1800) |
| ember-vale (3), brasswick (9), meridian-loop (10) | all fail | the same figures to 0.001 points; brasswick's frames are pixel-identical to Chrome's (RMSE 0), ember-vale's beacon within RMSE 0.00012 |
| gfx | not run here | og-tests-web does not build at cb2cd9216 (`depth_render_scenes`, fixed on main by f8564b8da) |

**Nothing differs because of Firefox.** The gameplay frames of brasswick
that moved against the first Chrome run (0.2 % at the pause button) moved in
Chrome too: 60a38f3ae dresses action buttons as discs, and the Metal frames
re-recorded at f0ab0f43d draw the disc as well (`brasswick-pause-disc.png`:
Chrome before 60a38f3ae, Chrome and Firefox after, and the new metal
reference). Firefox boots each station about 2.5 times slower than Chrome
(75 s against 31 s a station in fx-e5), with the same frames; where the
time goes was not measured.

a11y and lane-panic read the page through CDP (Chrome's accessibility tree
and the Worker's DevTools target) and have no BiDi path; they run in Chrome
and Edge.

## The wire, a11y and lane-panic at origin/main fa1332b06

Rebuilt in full at fa1332b06 (wasm, wasm-opt, `OG_GATE=1 build.sh`), with
the web boot thread's fixes on main (lane-panic's hashed Worker a9aa00a7d,
the custom UI material 853e5fde6, the paced warm-up e65899cce). Each run
on a fresh (cold) browser profile.

| Test | Chrome 156 | Edge | Firefox 159 (BiDi) |
|---|---|---|---|
| a11y | 26 of 26, twice | the contrast capture answers null in 2 of 3 runs (Worker gave up after 60 frames with the readback not landed on a cold D3D12 cache; passes warm); with web boot's time-based capture wait (ee99fcd63) never null, but the reduced-motion check fails 2 of 3: Edge flips `.matches` at once and fires `change` only at its next rendering update, after the harness's frames | no BiDi path (reads Chrome's accessibility tree) |
| lane-panic | 23 ok | not rerun | no BiDi path (the Worker's DevTools target) |
| wire.ts (325 checks) | 323: the clock moves 1→2 behind the gate in dev and publish isolated | 322: the same in publish isolated and the prewarm modes | 312: the same 1→2; a tap starts the clock after about 1.2 s (wire allows 600 ms) in the 4 sync modes; prewarm makes its 9 pipelines synchronously |

All of these are the web boot thread's and reported there with the logs.
The 1→2 is a settling tick: the store's answer to tick 0's request lands
after the first frame on a cold D3D12 profile and settle.ts applies it with
one tick (web boot's 6746421d0, not on main yet, allows it within SETTLE_MS). The rest has one cause in
Firefox: `queue.onSubmittedWorkDone()` answers about 100 ms after an empty
submit on an idle device there (118, 101, 100, 2459, 101 ms), so the warm-up
paced on it (one wait per pipeline, e65899cce) has made few pipelines when
the gate's tap comes, and the tap's frame makes the rest; web boot bounds
the wait. a11y's reduced motion is a harness fix (`emulate()` waits two
animation frames, web boot's aec72ed1c, not on main yet).

Firefox's wire runs as a local BiDi copy of wire.ts (bidi.ts: input to a
cross-origin frame goes to that frame's own context, which Firefox
requires).

## Rerun at origin/main a09b2ffdd

Rebuilt in full at a09b2ffdd, which has web boot's fixes (the 20 s capture
wait, a11y's `emulate()`, settling ticks in wire, three warm calls in flight;
5408c7e3d) and the terrain first-frame fix (e41b52272).

| Test | Chrome 156 | Edge | Firefox 159 (BiDi) |
|---|---|---|---|
| wire.ts (397 checks) | 396, then 397 twice | 397 | 396, then 397 |
| a11y | not rerun | 26 of 26, 3 of 3 cold profiles | no BiDi path |

- The one failure in Chrome and Firefox was the first mode of the first run
  after the new build: "dev isolated: a tap on the gate starts the clock"
  (tick 2 before and after the 600 ms wait). Reruns pass. With the tap timed,
  Chrome moves the clock 104-123 ms after the tap in every mode. Firefox
  takes 569 ms in dev isolated, 116-176 ms in the other sync-pipeline modes,
  and about 2.2 s in dev async. A cold driver shader cache on the first boot
  of a new build would explain it; that is not shown (NVIDIA's DXCache was
  locked and could not be set aside).
- The settling tick is what web boot read: every boot logs `[og] behind the
  gate: 1 settling tick(s) applied answers that landed after the first frame`,
  about 0.5-0.7 s after the first frame, and the clock then holds at 2.
- Firefox: sync's tap now lands in 117-119 ms, and prewarm compiles through
  createRenderPipelineAsync.

Terrain first frame (renderMs, reference-gfx), before (fa1332b06) and after:

| Scene | Chrome | Firefox |
|---|---|---|
| terrain-tiling | 33.5 s → 7.1 s | 25.6 s → 5.9 s |
| terrain-near | 36-39 s → 7.4 s | out of the capture budget 3 of 3 → 8.4 s |
| terrain-layers | 36.0 s → 7.4 s | 42.6 s → 8.2 s |
| terrain-mid | 21.3 s → 0.9 s | 29.8 s → 6.7 s |

Native debug og-gfx at a09b2ffdd: DX12 terrain-tiling 6.6 s, terrain-near
12.2 s (35.5 s and 34.4 s at 512ba82a0); Vulkan 3.2 s and 9.7 s (23.5 s and
21.2 s). Every frame matches its reference.

Misses closed since the fa1332b06 run (the first two on compat-win-web-2):
- water-shallows and water-wake were D3D12's coarse `ddx` (native DX12,
  Chrome and Firefox alike). water.wgsl now uses fine derivatives, and all
  three match Metal (`../native/water-*-derivatives.png`).
- The kit's 21 painted 3D-scene HUDs are BC7 against Metal's ASTC: per-tier
  references in og-ui `references/dx12` and `references/webgpu`
  (`../native/kit-manageHud-bc7-astc.png`).
- hooks-screen-ao passes against its re-recorded Metal frame (5e5c43195) on
  DX12 and Vulkan (0 % outliers, RMSE 0.0086).

## Adapters and limits

`adapter-chrome.json`, `adapter-edge.json`, `adapter-firefox.json`: each
browser's adapter info, features and limits (high-performance, low-power
and default). Firefox 159 (wgpu in Gecko) leaves the info strings empty,
answers subgroup sizes 4–128, maxBindGroups 8 and maxBindingsPerBindGroup
u32::MAX, but fewer dynamic buffers per layout than Chrome (8 uniform
against 10, 4 storage against 8) and none of subgroups, clip-distances,
primitive-index, texture-formats-tier1/2.

## PC presentation (G1-7)

`g17-chrome.json`, `g17-firefox.json`: the browser behaviour the shell's
G1-7 code reads, in each browser. Both grant pointer lock with
`unadjustedMovement` (raw input) and fullscreen from a gesture, report
`code` for physical keys (KeyQ, KeyA, KeyZ, KeyW, KeyM, Digit1), and fire
the `(resolution: Xdppx)` query watch shell.ts re-arms when the device
pixel ratio changes. Chrome has `navigator.keyboard.getLayoutMap()`;
Firefox has none and the shell learns labels from key presses
(keyboard-layout.ts LayoutLearner). Chrome under DevTools emulation fired
the DPR change once for three ratios 1.5 s apart (the query matched each
time when polled); Firefox fired all three. The machine has one monitor,
so a real move between monitors was not tried.
