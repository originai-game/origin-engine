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
| fx-e5 (31) | 9 fail: baked, moon, physical, shaders, sky, volumetric, islands, terrain, water | stale metal frames: at each reference's own commit DX12 passes (baked 0.000 %, physical 0.023 %, shaders 0.001 %, sky 0.021 %, volumetric 0.007 %, islands 0.063 %, water passes at eec965067); moon and terrain's residue at theirs was the `fract(sin)` star and caustic hash 71e70a0a2 replaced afterwards |
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
31, looks 37, ui-kit 235, gfx 206, ember-vale 3, brasswick 9,
meridian-loop 10), a11y 22 of 23 on the same null capture, and
lane-panic's same harness failure. Nothing here is specific to Edge.

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
