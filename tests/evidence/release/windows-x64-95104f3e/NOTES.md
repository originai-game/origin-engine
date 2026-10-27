# Windows x64 validation of 95104f3e (F7, studio-win)

Base `95104f3e` (0.6.0, PR #3, PR #4, the five RedGPU ports), then
`origin/main` at `2901621b` (metal references of the five new scenes, no
engine code) merged in. Branch `windows-validate-95104f3e`. 2026-10-27.

## Host

- Windows 10 Pro 25H2, build 10.0.26200.9711; NVIDIA GeForce RTX 5060,
  driver 610.88 (32.0.16.1088, 2026-07-22), DX12.
- Rust 1.99.1 x86_64-pc-windows-msvc, cargo-nextest 0.9.149, Bun 1.4.2,
  Node 22.23.4, wasm-bindgen 0.2.129, Binaryen 133, sccache 0.14.0.
- Chrome 155.0.8059.41, Edge 155.0.4283.45. Firefox is not installed.
  WebGL2 in both is ANGLE on D3D11 (`ANGLE (NVIDIA, NVIDIA GeForce RTX
  5060 Direct3D11 vs_5_0 ps_5_0, D3D11)`); WebGPU is Dawn on D3D12.

## Commits on the branch

| Commit | Fix |
|---|---|
| 8d66ba46 | og-cli: the 2D publish refusal names scenes with `/` on Windows (`a_2d_publish_build_refuses_the_3d_world_by_scene_and_line` failed: `Path::display` printed `assets\scenes\main.ogt:9`) |
| 136eeaca | og-tests: each gfx scene built in its own closure (the debug og-gfx stack overflow, below) |
| 726a9642 | web tests: Chrome's profile under `os.tmpdir()` (`TMPDIR ?? "/tmp"` became `C:\tmp`) |
| d3633590 | web tests: `reference.ts` used `split("/")` on a Windows path (EINVAL before Chrome started); `Browser.close` killed only the spawned process, leaving Chrome with its profile open (EBUSY, a stray headless Chrome after every run): now CDP `Browser.close`, kill after 5 s, EBUSY retried |

## The debug og-gfx stack overflow (root cause)

`og-gfx --all --capture` (debug) died with `thread 'main' has overflowed
its stack` in every scene that loads a glTF or vox model (19 scenes run one
at a time, plus terrain-generated). cdb (`sxe sov`, `kf`) on a
line-tables build of gltf-reference:

```
  Memory  Call Site
    7830  image::...::make_decoder           (zune-jpeg, the glTF's textures)
    1c80  og_render::gltf::load_glb
    24b0  og_tests::scenes::spike_gltf_scene
   c44e0  og_tests::scenes::build            <- 804 KB
   1cc20  og_gfx::run                        (117 KB)
```

`scenes::build` matched 137 scene names and built each inline. Unoptimised,
one frame holds every arm's temporaries side by side (`RenderScene` is
3,904 B, a `Result` of one as much again), so `build`'s frame was 804 KB.
On Windows' 1 MiB main thread that left too little for the image decoder.
At 0.2.0 the match had 94 arms and fit. nextest passed because its test
threads have 2 MiB. Nothing in the glTF/vox loaders grew; the growth is
the test harness's match.

The fix: the match picks one non-capturing closure per scene and calls it.
After the fix `build`'s frame is 12.5 KB (0x30c0) and the stack at the glTF
import is 157 KB, where it was 935 KB. No scene changed: all 132 DX12 frames
before and after are byte-identical. No committed test, because a reliable
one needs a 1 MiB thread and a debug build.

Players and the sandbox are not affected. The release og-gfx, built from
the old `scenes.rs` (95104f3e), rendered gltf-reference, showcase,
vox-keyframes, lod-levels and terrain-generated without overflow. The
shipped `og.exe` (release, 1 MiB reserve) ran `og run templates/fx-showcase`,
which has glTF models, and the 3D starter's tests in the install test.
Games run in bun.exe (18 MiB reserve) or node.exe (8 MiB), not on og's main
thread. og-tests is a dev-only crate.

## 1. Workspace

- fmt, `cargo check --workspace --all-targets`,
  `cargo clippy --workspace --all-targets -- -D warnings`: pass.
- `cargo nextest run --workspace --no-fail-fast` at 136eeaca: 1388 run,
  **1378 passed, 10 failed**, 18 skipped, 714 s. Every failure is a frame
  comparison:
  - The five known DX12-vs-metal differences, with the same numbers as
    `windows-x64.md`: atmosphere-night (RMSE 0.01650, 0.0914 %),
    atmosphere-moon (0.01613, 0.0896 %), sprites-array (0.02048,
    0.1360 %), tilemap-lit (0.03037, 2.7448 %), water-underwater (0.02975,
    3.8832 %).
  - Five that failed only for want of a metal reference at the time:
    post-adjust, post-blur, post-lens, post-neutral and terrain-grass-wind.
    Their metal references landed with 2901621b, and they match on DX12
    (§2).
  - The og-cli path failure is fixed by 8d66ba46.
  - Measured against the references, every failure is known. None is new.

## 2. og-gfx --all on DX12

`og-gfx --all --capture` (debug) rendered all 132 scenes in 321 s (msaa
does not apply on dx12). `scripts/frames-vs-metal.ts` against
`references/metal` at 2901621b ([`dx12-vs-metal.tsv`](dx12-vs-metal.tsv)):
126 meet the default rule (RMSE 0.01, 0.5 % over 32/255). Six are over it,
the coordinator's known six with identical numbers: atmosphere-night
0.01650/0.091 %, atmosphere-moon 0.01613/0.090 %, sprites-array
0.02048/0.136 %, stylise-pixel 0.01484/1.649 %, tilemap-lit
0.03037/2.745 %, water-underwater 0.02975/3.883 %.

The five new scenes on DX12:

| Scene | vs metal (2901621b) | vs vulkan-cpu |
|---|---|---|
| post-adjust | 0.009 %, RMSE 0.00465 | 0.053 %, 0.00633 |
| post-blur | 0.000 %, 0.00113 | 0.000 %, 0.00120 |
| post-lens | 0.007 %, 0.00410 | 0.008 %, 0.00470 |
| post-neutral | 0.000 %, 0.00134 | 0.000 %, 0.00122 |
| terrain-grass-wind | 0.019 %, 0.00354 | 0.020 %, 0.00583 |

All pass. I read each frame and each matches its NOTES description:
post-adjust has the warm sand floor and blue-violet robots; post-lens a
bowed horizon with no black corners; post-blur streaks towards the sharp
blue sphere; post-neutral keeps its orange sphere orange; in
terrain-grass-wind the tufts lean right.

## 3. og run templates/fx-showcase on DX12

The release `og.exe` ran the materials station at 1280x720: plain, Neutral,
colour adjustments (temperature 0.3, vibrance 0.6, contrast 0.25, teal
shadows), barrel lens 0.4, zoom blur 0.15 and directional blur 40 at 20°.
The run exited 0 in 12 s. I read each frame:

- Neutral: deeper blues and orange-yellow emissive.
- Adjust: a warm backdrop and richer gold.
- Lens: the rows bulge and the backdrop reaches the corners.
- Zoom: streaks towards the centre.
- Directional: streaks up to the right.

The HUD and og.ui badge stay sharp in every frame.

## 4. Release pack and install test

- `OG_RELEASE=1 sh packages/engine/scripts/pack.sh` at 136eeaca (clean
  tree): PASS in 1470 s.
  `origingame-engine-win32-x64-0.6.0.tgz` 36,675,054 B
  (sha256 33c634475f93e0616aa790a624efb466e6c6cf07b46f5d9f3478f2924fc5e2bc),
  `origingame-engine-0.6.0.tgz` 25,029,820 B
  (da837eab083247f627de7b65d732ee6953043f3d5d4f8545dd7df4fc36f652e3),
  `origingame-create-game-0.6.0.tgz` 16,320,408 B
  (ae89bd25fb5b626d94fe19c5afd8399bf32d58668a67465ce623473d5c5d81c2).
  Local candidates only, not published.
- `bun tests/install/run.ts --from <those three> --version 0.6.0 --host-only
  --keep`: **PASS in 102 s** (3D starter 3 suites, runner archetype, 2D
  starter 2 suites, publish wasm byte-identical, knowledge validate,
  mount and check). Doctor:
  `renderer: ok dx12 (a 64x64 frame headless in 1206 ms)` and the
  `tsgo 7.0.2 through bun x` checker fallback. The smoke frame was read
  and renders normally.

## 5. Browsers (web-reference harness, release web build of d3633590)

| Lane | Result |
|---|---|
| Chrome WebGPU, fx-showcase stations | PASS 5/5 (post 0.00455, lens 0.00676, fx 0.00392, 2d 0.00288, stylised 0.00863 vs metal) |
| Edge WebGPU, fx-showcase stations | PASS 5/5 |
| Chrome WebGPU, og-tests gfx (132 scenes and og-ui's) | 126 of 132 pass. The six failures are the known DX12 six, with the same numbers to five decimals as native |
| Edge WebGPU, og-tests gfx | the same six failures, the same numbers |
| Chrome WebGL2 (ANGLE D3D11), fx-showcase | **FAIL**: the engine panics at tick 0 |
| Edge WebGL2, fx-showcase | **FAIL**: the same panic |
| Chrome WebGL2, og-tests gfx | **FAIL**: every scene fails |

### WebGL2 on Windows: new finding, not fixed

The WebGL2 tier does not start in either Chrome or Edge on Windows. Chrome's
log (`--enable-logging`) shows ANGLE's D3D11 backend translating the lit
fragment shader's `cluster_table[c >> 2u][c & 3u]` and
`cluster_refs[...][(j >> 2u) & 3u]` (crates/og-render/src/clusters.rs).
That is a `vec4<u32>` indexed by a variable, and ANGLE emits its helper
`dyn_index_uvec4_uint4_int` once per use. FXC then refuses with
`error X3003: redefinition of 'dyn_index_uvec4_uint4_int'` at the first
draw (`Context11::triggerDrawCallProgramRecompilation`), and the context
is lost. The next pipeline wgpu makes (`sprite`) then fails with an empty
"Internal error". This was reproduced by linking the lit shader alone in
a page.

The code dates from the Forward+ commit a0611133 (2026-10-19, before
0.1.0), so it is not a regression of these ports. No earlier Windows
WebGL2 run exists in the evidence: the Mac's ANGLE uses Metal, and the orb
uses GL/SwiftShader, so neither runs this path.

I tried one fix: picking the lane with `select` chains, or a bit mask, in
place of `v[i]`. It removes the X3003, but FXC then takes **128 s** to
link that one shader (measured alone; the page times out), so it was
reverted and not committed. A fix needs a different layout, for example
the tables as `array<u32>` packed so that no component index is dynamic.
That is a renderer design decision for the coordinator.

Not run: Firefox (not installed); WebGL2 frames on Windows (blocked by the
above); Edge WebGL2 gfx (same failure as Chrome expected; not run).
