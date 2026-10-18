# The publish wasm, E7 (2026-10-19)

`og_host_web-publish` measured as `scripts/check.sh size-gate` does:
`--features og-host-web/publish`, wasm-bindgen, `wasm-opt -Oz
--enable-bulk-memory --enable-nontrapping-float-to-int --low-memory-unused`,
`brotli -q 11`, against `wasm_core_brotli_bytes` 4,194,304. Every row is a
full build of the tree at 96f4ebd with only the named change; Apple M4 Pro,
another agent on the machine.

Speed is measured two ways, both against the release (opt-level 3) build:

- **physics**: `tests/support/physics-hash.ts --host web --ticks 1200`
  (64 bodies on a ramp, a paddle, a sensor), the advance's milliseconds, three
  runs, 3D and 2D. The world hash is the same on every row (3D
  `6cebf6f8086736a5`, 2D `42644716f028884c`): no variant changes a result.
- **frames**: Chrome for Testing 150, headless, 1280×720, reference.ts's
  page driving one `frame` of one tick and one render per call (no
  captures), the median per-frame milliseconds over fx-showcase's stations
  (480 frames), ui-kit's main menu and runner (300 each), two to four runs
  on WebGPU and on WebGL2, plus the first frame (pipelines and shaders
  created) and the boot to a started engine.

## What was tried

| Change | brotli | of budget | Δ | Speed | Kept |
|---|---:|---:|---:|---|---|
| Release, opt-level 3 (before) | 3,021,619 | 72.0 % | | physics 3D 285–287 ms, 2D 211–221 ms; frames WebGPU fx 0.62–0.69, ui-kit 0.48–0.53, runner 0.67–0.71; WebGL2 fx 1.32–1.43, ui-kit 0.74–0.80, runner 0.98–1.01 | |
| `--remap-path-prefix` (shorter panic paths) | 3,021,586 | 72.0 % | -33 | | no: nothing |
| `lto = "fat"` | 3,020,227 | 72.0 % | -1,392 | | no: twice the link time for 0.05 % |
| wasm-opt `-Oz --converge` | | | -3,420 | | no: 0.1 % for a minute more per gate |
| wasm-opt `-Oz -Oz` / `--gufa -Oz` | | | -184 / +2,094 | | no |
| opt-level `s` everywhere | 2,629,821 | 62.7 % | -391,798 | physics 3D 463–471 ms (+63 %), 2D 301–308 ms (+43 %) | no: the physics step |
| opt-level `z` everywhere | 2,297,886 | 54.8 % | -723,733 | physics 3D 643–650 ms (+125 %), 2D 425–432 ms (+102 %) | no: the physics step |
| `z` everywhere but physics, ECS, og-core, og-anim, og-render and wgpu at 3 | 2,840,416 | 67.7 % | -181,203 | physics +3 %; frames WebGPU fx 0.83–0.85 (+25 %), ui-kit 0.70–0.71 (+35 %), runner 0.90–0.94 (+33 %); WebGL2 +8–20 % | no: every frame |
| `z` for load-time crates (below) plus og-probe, serde_json, og-api, og-nav, og-tilemap, og-ai | 2,939,196 | 70.1 % | -82,423 | frames as before but WebGPU runner 0.72–0.75 | no: og-nav, og-ai and og-tilemap run every tick, unmeasured here |
| the same without og-probe, serde_json, og-api | 2,936,481 | 70.0 % | -85,138 | frames as before | (the kept set is this minus og-nav, og-tilemap, og-ai) |
| **`[profile.wasm]`: release, load-time crates at `z`** | **2,943,794** | **70.2 %** | **-77,825 (-2.6 %)** | physics 3D 250–265 ms, 2D 177–194 ms; frames WebGPU fx 0.65–0.67, ui-kit 0.51–0.53, runner 0.73–0.74 (0.02–0.07 ms more, the one case outside the runs' spread); WebGL2 fx 1.32–1.36, ui-kit 0.76–0.77, runner 0.97–1.00; first frames 13.9–20.0 ms WebGPU, 14.9–45.4 ms WebGL2 (as before) | **yes** |

The kept set, every one of them work done while loading or when something
changes, never per tick or per frame: naga, naga-types, wgpu-naga-bridge and
codespan-reporting (WGSL translation for WebGL2 and its errors, at pipeline
creation), rustybuzz, ttf-parser and the unicode tables (shaping, when a
text changes), og-cook, og-blob and miniz_oxide (opening the cooked bundle),
bincode (snapshots), spade and rstar (navmesh triangulation), og-i18n (locale
tables). Its gate run: `wasm-build`, `wasm-opt` and `size-gate` PASS at
2,943,794 B (70 %); `web-reference` matches all 26 browser frames (fx-showcase's
five and eight E5 stations on WebGPU and WebGL2) at the RMSE they had before.

Two larger single-crate steps were measured and left at 3: og-runtime at `z`
saves another 71,670 B (its per-tick systems and the frame's dispatch run in
it) and wgpu-core, wgpu-hal, glow and wgpu-types another 53,262 B (the
WebGL2 tier's per-frame command path).

## What the bytes are

Of the raw wasm's 12.0 MB of function bodies, physics is the largest part:
og-physics 7.2 %, rapier3d, rapier2d, parry3d and parry2d with their
solvers and soft bodies about 15 % more, including about 150 KB of serde
deserialisers for the physics state block (snapshots). og-runtime 9.5 %,
wgpu-core and wgpu-hal 3 %, naga 4 % with its lowerer and validator,
rustybuzz and ttf-parser 4 %. The data section is 1.12 MB raw, 282 KB
brotli: the IDL's field docs are 41.8 KB of it (12.3 KB brotli, 0.3 % of the
budget, which the runtime's reflection serves), WGSL with its comments about
50 KB, panic locations 74.8 KB raw (removing them needs nightly's
`-Zlocation-detail=none`).

## What going further takes

Below 65 % honestly needs the publish wasm per profile. A game's profile
picks one simulation (og-physics keeps `sim3` or `sim2`, never both), and
og-physics already has the `physics2d` and `physics3d` features. Built with
one of them (opt-level 3, no other change):

| Publish wasm | brotli | of budget |
|---|---:|---:|
| both profiles (today) | 3,021,619 | 72.0 % |
| 3D only | 2,654,939 | 63.3 % |
| 2D only | 2,496,653 | 59.5 % |

With the wasm profile on top, about 61 % and 57.5 % (not measured together).
It takes: og-runtime and og-host-web passing the two features through; two
publish variants in `scripts/wasm-crates.txt` and the gate; `dist/web-publish-2d`
and `-3d` in `packages/engine` (`build.sh`, `pack.sh`, `package.json`'s files)
and `og build --publish` choosing by `GameDesc`'s profile; a refusal when a
game's profile is not in the wasm; and the size budget read per download
(each publish wasm ≤ 4 MB) instead of as the sum of every wasm the gate
builds, a reviewed change to `scripts/budgets.txt`. It was not done in this
round: it changes what `packages/engine` ships and what the budget means.
Past that, the per-tick crates at `s` or `z` (above) cost a quarter of a
frame or the physics step half again.
