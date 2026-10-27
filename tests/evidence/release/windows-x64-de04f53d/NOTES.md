# Windows lane on de04f53d (the 0.6.1 candidate, studio-win)

Base `de04f53d`, which was `origin/main` when the run started and when it
ended (19:00–19:50 NZDT): the debug og-gfx stack fix, the web harness's
Windows fixes, and the merge of `webgl2-cluster-table` (b6340a10). Branch
`windows-lane-de04f53d`. 2026-10-27. docs/release.md §T, the windows row:
the DX12 frames and the browsers. The native workspace check, clippy and
nextest, and the install test from tarballs, were not part of this run
(the install test follows on the release commit).

## Host

- Windows 11 Pro 25H2, build 10.0.26200.9711 (the registry's ProductName
  reads "Windows 10 Pro", which earlier notes copied); NVIDIA GeForce RTX
  5060, driver 610.88 (32.0.16.1088, 2026-07-22).
- Chrome 155.0.8059.41, Edge 155.0.4283.45. WebGL2 in both: `ANGLE
  (NVIDIA, NVIDIA GeForce RTX 5060 (0x00002D05) Direct3D11 vs_5_0 ps_5_0,
  D3D11)`; WebGPU: Dawn on D3D12.
- Rust 1.99.1 x86_64-pc-windows-msvc, Bun 1.4.2, Node 22.23.4,
  wasm-bindgen 0.2.129, Binaryen 133.
- No headless Chrome, Edge, og-gfx or harness process was running before
  the run, and no `og-cdp-*` profile was left in `%TEMP%`.

## Commits on the branch

| Commit | What |
|---|---|
| d0588b5a | web tests: `cdp.ts` polls `DevToolsActivePort` in a `try`. On Windows, reading it while Chrome still had it open threw EBUSY out of `Browser.launch`, which left the just-spawned browser running: 2 of about 25 launches here (the Edge link-time run, whose orphaned Edge was 14 headless processes, and a probe). 16 launches in a row after the change. |

Not on this branch: the og-ui/glass fix, `f102a6e3` on branch
`webgl2-glass-pyramid-size` (off `de04f53d`), below. It changes og-render,
so taking it moves the release commit.

## 1. og-gfx --all on DX12 against metal

`target/debug/og-gfx.exe --all --capture`, then `bun
scripts/frames-vs-metal.ts --set dx12 … references/metal`: 132 frames in
284 s, msaa does not apply on dx12. Table: `dx12-vs-metal.tsv`.

All 132 rows (outliers and RMSE) are identical to
`../windows-x64-95104f3e/dx12-vs-metal.tsv`. Over the rule (RMSE ≤ 0.01,
outliers ≤ 0.5 %) only the six known differences, with the same numbers:

| Scene | Outliers | RMSE |
|---|---|---|
| atmosphere-night | 0.091 % | 0.01650 |
| atmosphere-moon | 0.090 % | 0.01613 |
| sprites-array | 0.136 % | 0.02048 |
| stylise-pixel | 1.649 % | 0.01484 |
| tilemap-lit | 2.745 % | 0.03037 |
| water-underwater | 3.883 % | 0.02975 |

Nothing new.

## 2. web-reference in Chrome and Edge

The gate's wasm (`scripts/check.sh --only wasm-opt`, 237 s: wasm-build
233 s, wasm-opt reused byte-identical modules) and `OG_GATE=1 sh
packages/engine/scripts/build.sh`, then from `packages/engine`
`tests/web/reference.ts` (fx-showcase) and `tests/web/reference-gfx.ts`
(og-tests' and og-ui's scenes), each `[--webgl] --chrome <exe>`. Every
per-scene number: `web-scenes.tsv`.

fx-showcase, 5/5 in all four lanes:

| Station | WebGPU (Chrome = Edge) | WebGL2 (Chrome = Edge) |
|---|---|---|
| post | 0.019 % / 0.00455 | 0.019 % / 0.00675 |
| lens | 0.030 % / 0.00676 | 0.030 % / 0.00675 |
| fx | 0.023 % / 0.00392 | 0.040 % / 0.00469 |
| 2d | 0.008 % / 0.00288 | 0.008 % / 0.00290 |
| stylised | 0.046 % / 0.00863 | 0.046 % / 0.00864 |

Stations: Chrome WebGPU 7.9 s, Edge WebGPU 8.6 s, Chrome WebGL2 60.2 s,
Edge WebGL2 60.7 s.

gfx scenes:

| Lane | Rendered | Pass | Fail | Not on the tier | Time |
|---|---|---|---|---|---|
| Chrome WebGPU | 131 | 125 | 6 | msaa | 91.7 s |
| Edge WebGPU | 131 | 125 | 6 | msaa | 97.1 s |
| Chrome WebGL2 | 128 | 121 | 7 | taa, shadows-local-8, sprites-untonemapped, hooks-planar | 249.7 s |
| Edge WebGL2 | 128 | 121 | 7 | the same | 251.7 s |

The failures in every lane are the six DX12 differences above with the
same numbers; the seventh on WebGL2 is og-ui/glass (0.000 % outliers,
RMSE 0.01728; §3). Every number in all eight lanes is identical to the
b6340a10 run (`../windows-x64-webgl2/NOTES.md`). Nothing new.

## 3. og-ui/glass on WebGL2: an ANGLE D3D11 bug, fixed in og-render

Frames: `glass-webgl2.png` (Chrome and Edge give the same bytes),
`glass-webgl2.diff.png`, `glass-webgpu.png`, against
`crates/og-ui/references/metal/glass.png`.

Where: only inside the glass panel. Outside it the WebGL2 frame equals
metal's exactly (mean |d| 0.000); inside, mean |d| 8.67/255, max 26.
WebGPU's frame differs from metal's by at most 1/255. The orange bar
under the panel is blurred about twice as wide: column x = 120, RGB,

| y | metal | WebGPU | WebGL2 | WebGL2, fixed |
|---|---|---|---|---|
| 40 | 187,187,187 | 187,187,187 | 191,187,184 | 187,187,187 |
| 70 | 190,187,185 | 190,187,185 | 195,187,180 | 190,187,185 |
| 86 | 212,185,158 | 212,185,158 | 202,186,171 | 212,185,158 |
| 100 | 220,184,145 | 220,184,145 | 205,185,167 | 220,184,145 |
| 114 | 210,185,160 | 210,185,161 | 202,186,172 | 210,185,161 |
| 140 | 188,187,187 | 188,187,187 | 194,187,181 | 188,187,187 |

Why: og-render's glass pyramid (src/glass.rs) draws each of its 6 levels
from the one above. wgpu's GL backend samples a level through
`TEXTURE_BASE_LEVEL` = `TEXTURE_MAX_LEVEL` = that level, and glass.wgsl's
`fs_down` spaced its taps by `1 / textureDimensions(source, 0)`. ANGLE on
D3D11 answers `textureSize(s, 0)` with base level b as level 2b's size
(`angle-texture-size.ts`, a 64-px texture):

| Base level | Chrome, Edge (ANGLE D3D11) | ANGLE GL | ANGLE Vulkan | Right |
|---|---|---|---|---|
| 0 | 64 | 64 | 64 | 64 |
| 1 | 16 | 32 | 32 | 32 |
| 2 | 4 | 16 | 16 | 16 |
| 3 | 1 | 8 | 8 | 8 |

(Its translated HLSL calls `GetDimensions(baseLevel, …)` on the bound
texture.) So levels 0 and 1 are right and the taps of every later level
are 2×, 4×, 8× too far apart; the panel's blur 16 reads level 3.
`angle-glass-pyramid.ts` builds the same pyramid in raw WebGL2 both ways,
one texture with base levels as wgpu does and one texture per level: on
ANGLE D3D11 the levels differ by `[0, 0, 35, 34, 24, 16]`/255 (Chrome and
Edge); on `--use-angle=gl` and `--use-angle=vulkan` by 0. ANGLE Metal on
the Mac draws og-ui/glass at RMSE 0.00000 (`../mac-95104f3e`).

Verdict: a defect, caused by an ANGLE (D3D11 backend) bug, visible to
players: glass panels in WebGL2 on Windows blur too wide and too faint.
Not a driver difference and not acceptable as tolerance.

Fix, on branch `webgl2-glass-pyramid-size` (`f102a6e3`, off `de04f53d`,
not on this branch): the first level's uniform carries the source's size
and each later level gets a uniform with the size of the level it reads;
`fs_down` divides the same integers as before. Checked here: og-ui/glass
WebGL2 in Chrome and Edge RMSE 0.00035 against metal (`glass-webgl2-fixed.png`;
WebGPU's is 0.00035 too), WebGPU unchanged, og-ui's other three frames
unchanged on WebGL2 (sample-screen 0.00214, text-glow 0.00037,
widget-set 0.00133), the DX12 glass frame byte-identical to de04f53d's,
og-render tests (89 + 13 + 1), clippy, fmt, og-ui's gfx tests. No tolerance
or reference changed.

Same bug, not fixed: the view glow chain (views_glow.wgsl through
bloom_chain.wgsl's `downsample13` and `bloom_tent`) reads
`textureDimensions` of a level view of one texture the same way, so a
view's og.Glow on WebGL2 on Windows should blur too wide as well. No web
scene draws it (only og-tests' native targets test), so it is inferred,
not seen. The frame's own bloom uses one texture per level and is not
affected. ibl, mipmap and pack use no `textureDimensions` on level views.

## 4. WebGL2 link times and fx-showcase's cold start

Every GL-tier entry point at this commit (256, the GLSL og-render's
`tests/shaders.rs` writes for WebGL2) compiled and linked in a fresh
headless page, then drawn once (the windows-x64-webgl2 method).
Per entry point: `webgl2-link-times.tsv`.

- Chrome: all 256 link, no lost context; 184.1 s in all. Edge: the same,
  183.7 s.
- Lit fragment shaders, Chrome / Edge: pbr 9.6 / 9.7 s (fs), 10.8 / 10.1 s
  (fs_lean), 6.4 / 6.8 s (fs_lightmap); pbr array maps 9.7 / 11.0 s,
  10.0 / 9.7 s (fs_standard), 8.7 / 9.3 s (fs_lean); pbr lean 9.7 / 9.9 s;
  physical 10.4 / 10.9 s; stylise object 9.7 / 9.4 s, 10.3 / 8.9 s
  (fs_lean); terrain 8.3 / 7.7 s, terrain (layer arrays) 8.5 / 8.4 s.
  So 6.3–10.8 s in Chrome and 6.8–11.0 s in Edge, as known (8–11 s).
- Others over 1 s: scatter 4.7 / 3.8 s, custom lit 4.4 / 3.6 s, fluid 3d
  composite 3.8 / 3.6 s, volumetric raymarch 1.7 / 1.6 s.
- fx-showcase's cold start (the reference JSON's pipelines made behind
  the gate): WebGL2 37 pipelines in 25.8 s (Chrome) and 26.7 s (Edge), as
  known (25 s); WebGPU 43 pipelines in 0.60 s and 0.70 s (1.3 s and
  1.5 s in the b6340a10 run; a warm driver shader cache is the likely
  reason, not checked).

Owed follow-up as before, not blocking.

## 5. Versions

Windows 11 Pro 25H2 build 26200.9711, NVIDIA driver 610.88
(32.0.16.1088), Chrome 155.0.8059.41, Edge 155.0.4283.45.
