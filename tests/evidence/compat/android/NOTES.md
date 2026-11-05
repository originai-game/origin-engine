# Android compatibility: Chrome on the owner's Pixel

studio-win (host "fan", Windows 11 26300), base origin/main c00851efc.

## Device

- Pixel 10 (`frankel`, serial 58051FDCR007QL), Android 17
  (CP3A.261005.005), Tensor G5. Its GPU is **Imagination PowerVR
  D-Series DXT-48-1536** (GLES "build 25.3@6908880"; WebGPU `info`
  vendor `img-tec`, architecture `d-series`), not Adreno or Mali: the
  Adreno and Mali rows of AGENTS.md's compatibility phase need other phones.
- Chrome 156.0.8078.79 (Android); desktop control: Chrome 156.0.8078.80 on
  an NVIDIA Blackwell (D3D12) on the same machine.
- Screen 412x924 CSS px at DPR 2.625; 8 cores; `deviceMemory` 8;
  MemTotal 11.3 GiB.

## Setup

- adb: Android SDK Platform-Tools 37.0.1 (`winget install
  Google.PlatformTools`; on the user PATH; a shell started before the
  install needs `%LOCALAPPDATA%\Microsoft\WinGet\Packages\Google.PlatformTools_Microsoft.Winget.Source_8wekyb3d8bbwe\platform-tools`
  on PATH). The phone first listed as `unauthorized`; the owner allowed
  USB debugging with "Always allow".
- The runners reach the phone with `--android [<serial>]`
  (`packages/engine/tests/web/cdp.ts` `Browser.android`): the DevTools
  socket on a forwarded port (`adb forward tcp:0
  localabstract:chrome_devtools_remote`), each test-server port reversed
  (`adb reverse`), so the page's 127.0.0.1 is this machine's and a secure
  context. A run opens its own tabs in the phone's Chrome, closes only
  those, and removes its forward and reverses. The page is driven at the
  runners' size with `Emulation.setDeviceMetricsOverride` (1280x720, DPR 1:
  the phone reports `devicePixelRatio` 1.0000000186).
- Tiers: the phone's frames are `webgpu-android`, judged against that
  tier's reference, else `webgpu`, else `metal`. No `webgpu-android`
  reference is recorded unless a row below calls the difference legitimate.

## WebGPU adapter (`adapter.json`, phone beside desktop Chrome)

`requestAdapter({ powerPreference: "high-performance" })` on both;
`deviceDefaultLimits` is what a plain `requestDevice()` gets.

- Preferred canvas format: phone **rgba8unorm**, desktop bgra8unorm. The
  engine picks an 8-bit format from the surface's capabilities
  (og-render `renderer/init.rs`) and its capture swizzles BGRA
  (`capture.rs`), so this is handled.
- Features only the phone has: `texture-compression-astc`,
  `texture-compression-etc2`. Only desktop: `texture-compression-bc`(-sliced-3d),
  `float32-filterable`, `bgra8unorm-storage`, `atomic-vec2u-min-max` and
  Chromium's experimental `multi-draw-indirect`,
  `sampling-resource-table`, `timestamp-query-inside-passes`. Both:
  `subgroups` (phone 32-128 wide, desktop 32), `shader-f16`,
  `timestamp-query`, `dual-source-blending`, `float32-blendable`,
  `rg11b10ufloat-renderable`, `indirect-first-instance`,
  `texture-formats-tier2`.
- WGSL language features: the phone lacks `fragment_depth`.
- Limits that differ: `maxStorageBufferBindingSize` phone 128 MiB
  (desktop 2 GiB), `maxBufferSize` phone 4 GiB (desktop 2 GiB),
  `maxVertexAttributes` 16 (30). The engine requests
  `Limits::downlevel_defaults()` (128 MiB storage bindings, 16 vertex
  attributes) raised to eight storage buffers a stage, all within the
  phone's limits.

- `indirect-first-instance`: exposed on the phone (and desktop), so the
  GPU-driven crowds (b1046f066's indexed indirect draws) get the feature
  they need; their frames are in the run below.

## Triage (first pass, in progress)

Base origin/main 012a073a3 (the Windows thread's wasm and package build,
copied read-only), then the fix on b1046f066. Frames under `frames/`.

| Run | Result | Reading | Class |
|---|---|---|---|
| fx-showcase stations (reference.ts --android --time 120) | 1 of 5 | `2d` passes (0.12 % outliers, RMSE 0.0094 against metal). `post`, `lens`, `fx`, `stylised` are white across the whole frame, HUD included (96-99.5 % outliers). The engine log: Dawn's `CreateGraphicsPipelines failed with VK_ERROR_UNKNOWN` for the RenderPipeline "meshlet resolve", then every `frame` command buffer invalid. | engine bug, fixed (below) |
| fx-showcase frame time, before the fix | median 5.53 ms, p95 8.33 ms, 6.17 ms with the GPU's work | Not meaningful while every frame's commands are invalid; retaken after the fix. | - |
| The fallback on the web: wasm panic | boot panics at wgpu `backend/webgpu.rs:85` "Unexpected error" | wgpu 30's WebGPU backend decodes only GPUValidationError and GPUOutOfMemoryError from popErrorScope (gfx-rs/wgpu#9914, fixed upstream by #9919, unreleased); the resolve's `internal` scope pops PowerVR's GPUInternalError. Fixed in the Worker, wgpu used as published (985cf7e71, `src/web/gpu-errors.ts`; `tests/web-gpu-errors.test.ts` fails without it). A first fix vendoring wgpu (131dba771) broke AGENTS.md and is reverted (454a07f99). | engine bug, fixed; confirmed on the phone (below) |
| The Worker fix on the phone (100183b21's build on f0ab0f43d, crates.io wgpu) | no panic; 1 of 5 | First frame 3.46 s. `post` and `lens` ok; `fx` 0.163 % / RMSE 0.0118, `2d` 0.212 % / 0.0138, `stylised` 0.415 % / 0.0145 miss narrowly, white frames gone. Before the tap `capabilities.visibilityBuffer` reads true (the resolve pipeline is made at first use); after it, false, with `visibilityBufferRefused` "internal error: CreateGraphicsPipelines failed with VK_ERROR_UNKNOWN - While initializing [RenderPipeline "meshlet resolve"]". | fixed; the narrow misses open until the desktop Chrome control |
| fx-showcase stations after the fallback (131dba771's build, the same Worker-side effect) | 1 of 5; no white frames | `post` ok (0.014 %, RMSE 0.0042). `lens` 0.27 % / 0.0124, `fx` 0.16 % / 0.0118, `2d` 0.21 % / 0.0138, `stylised` 0.70 % / 0.0184: the frames match metal's by eye (stylised's outlines and toon bands differ a little). `capabilities.visibilityBuffer` false, `visibilityBufferRefused` the driver's VK_ERROR_UNKNOWN. | open: compare with desktop Chrome on the same build before calling any of it legitimate |
| og-ui kit tiers (reference-ui.ts), first run | 1 of 233 | Every kit frame laid out touch-sized: the phone's tab reports `(pointer: coarse)`, the shell sends `coarsePointer`, og-ui and the kits size for fingers. The runners' references are a desktop's input, and CDP's `setEmulatedMedia` does not change pointer or hover on a touch device. | harness issue, fixed (45b7583b7: phone tabs answer as a mouse unless opened with `touch`) |
| og-ui kit tiers, rerun | 11 of 233 (median 4.1 % outliers) | Run on a checkout based on b1046f066, whose metal kit references predate main's: the frames- references were re-recorded on Metal at 0f2ab52a0 (59/59 at fc9a08bb6, after 60a38f3ae's disc controls), and the style-fantasy and style-scifi sheets still hold 27 cells for 33 shots on main (being re-recorded). Rebased onto 5d4e5b8dc; rerun in the next build window, after the style sheets land. | stale references in this checkout, not Android: rerun pending |
| Boot (fx-showcase test-reference export) | first frame 3.6-3.9 s | wasm 1.07-1.17 s, bundle 1.35-1.46 s, transcode 0.41-0.52 s, present 0.12-0.17 s; 66 pipelines warmed behind the gate in 0.7 s. | - |
| A stall after the tap | tick frozen 9-11 s, every run, warm cache | The warm-up behind the gate compiles after the tap: a Worker hook shows every pipeline (109 render, 17 compute) and shader module created before the tap and none created or first used after it, yet Chrome's trace has the GPU process create 65 render pipelines from 0.08 to 11.7 s after a tap 20 s after the first frame (11.5 s in Dawn: terrain 2.67, pbr 1.94, sky 1.19, water 0.92, ...). The commands wait unflushed in the wire until the tap's first submit, and PowerVR then compiles them serially (128 pipelines, about 24 s of Dawn time in all, despite Dawn cache hits). An empty submit after each pipeline (scratch hook) leaves 14 after the tap (3.7 s) and the tick frozen 4.2 s. Handed to the web-boot thread (T-01a1a707), which owns the warm-up; its async/prewarm/sync comparison on the phone follows. | engine issue, owned by T-01a1a707 |
| G1 frame cap | inconclusive | fps60 and fps30 both measured 26 fps at a 24 Hz display interval: the windows fell in the stall above. Rerun. | open |
| G2 render scale 0.5 | works | scene 412x858 in an 823x1715 canvas; 48.7 fps, work 7.6 ms. | ok |
| G3 battery saver | works | capped at 30 (30.1 fps, interval 33.2 ms), scale 0.75. | ok |
| G4 auto quality (default settings) | works | from scale 1 at 30 fps it stepped 0.9 → 0.5 and one preset step in 8 s and held 52 fps at 60 Hz (frame 23-24 ms of work and GPU). At render scale 2 for 95 s: 48-49 fps. | ok |
| Thermals (95 s at render scale 2, auto on) | status 0 throughout | skin 31.9 → 36.4 °C, SoC 36.2 → 43.8 °C, battery 30.1 → 36.0 °C; no throttling seen. | - |
| Memory (fx-showcase) | | wasm linear 77 MiB (heap live 63 MiB, peak 67 MiB); the engine's GPU count 146 MiB (render targets 131 MiB); Chrome's PSS 338 MiB, Graphics 70 MiB, EGL mtrack 51 MiB. | - |

### The meshlet resolve on PowerVR

The visibility buffer's lit resolve (`meshlet.wgsl` `fs_meshlet_resolve`,
the lit module with `ANALYTIC` true; `crates/og-render/src/meshlet_gpu/vis.rs`)
is a valid pipeline that the PowerVR driver refuses. Reproduced outside
the engine with the captured module on a bare device in the phone's Chrome
(`createRenderPipelineAsync`, layout auto, rgba16float target,
depth32float):

- refused: the entry as it is; with `EFFECTS` false; without its depth
  write; without its discard; with any single attribute of the record kept
  and the rest constant; with `world_dx`/`world_dy` returning constants.
- accepted: `ANALYTIC` false; the record's vertex replaced by constants;
  the record's `analytic_uv`, `analytic_dx` or `analytic_dy` store removed
  (any two of the three, or `dx` and `dy` alone, still refused); a minimal
  shader with the same dependent loads and cross product.

So no single construct is at fault: the driver's compiler does not fit the
whole shader once its gradients cannot be folded. The engine's bug was what
followed: drawing with the refused pipeline invalidated the frame's command
buffer, so one optional pipeline whitened every 3D frame. Fixed at the root
in og-render (661c1711e): the resolve's pipelines are made inside
validation and internal error scopes, and a refusal turns the visibility
buffer off for that device, its clusters drawn on the hardware meshlet path
as with `RenderScene::visibility_buffer` off. Its unit test
(`a_resolve_pipeline_the_device_refuses_turns_the_visibility_buffer_off`)
panics on the old code (wgpu's uncaptured validation error) and passes now.

Not run yet on the phone: reference-gfx (the Safari cluster first: terrain, water, grass, impostor-scatter, spheres-20k, with and without `--cpu-culling`; crowd-lods and the crowd-2k sweep), e5, wire.ts and the templates' playtests. reference-gfx needs a wasm build window, shared with the rebuild for 985cf7e71.

### The warm-up behind the gate on PowerVR (for the web-boot thread, T-01a1a707)

fx-showcase dev export from a25fc9ebf + the resolve fallback + the Worker's
GPUInternalError fix + 4f2d99959 (each warm call flushed and awaited one at
a time), warm Chrome cache, Pixel 10. "Screen" is a CDP screenshot
requested at first-frame + 1 s: it returns only once the page makes a
compositor frame.

| | sync | prewarm | async |
|---|---|---|---|
| og:first-frame from navigation | 3.7-4.4 s | 3.8-4.3 s | 17.8-17.9 s |
| screenshot at ff+1 s | 22.7 s (next frame at ff+23.7 s) | 22.7 s (ff+23.7 s) | 96 ms; 96-152 ms through ff+42 s |
| tap at 20 s: tick frozen / longest frame gap | 0 / 58 ms | 0 / 124 ms | 0 / 66 ms |
| tap at 1 s: tick moves after / frozen / gap | 0.29 s / 0.52 s / 0.85 s | 0.29 s / 0.03 s / 0.15 s | - / 0 / 67 ms |
| whole warm-up (tap at 40 s) | 128 pipelines, 35.9 s in Dawn, last ends ff+38.5 s | 128, 19.8 s in Dawn, ends ff+25.9 s | 128, 7.5 s on Dawn's main thread, ends ff+4.8 s |
| boot.pipelines 20 s after ff | sync 41 | sync 41, spans 25 | async 91, sync 16, fallback 2, spans 142 |
| longest HandleDawnCommands | 8.5-9.8 s at ff+0.6 (the first frame's pbr x2, pbr meshlet, meshlet resolve, sprites, ...), then 3.9-4.0 s | 8.7-11.1 s at ff+0.5, then 4.2-5.6 s | 5.2 s, 11.8 s before ff |
| pipelines compiled after a tap at 20 s | 1 (fs_gtao_bent, 0.05 s; the warm-up stopped at 74 of 109 render) | 0 | 0 |

The per-call flush works as meant (no freeze at a tap 20 s in, the tick
runs at a tap 1 s in), but on PowerVR Chrome compiles the sync pipelines,
and prewarm's here too, on the GPU process's main thread, so the page makes
no frame for 22.7 s after its first-frame mark, the gate included. Async
waits longer for its first frame and then never blocks: a live screen at
about 18 s from navigation against about 27.5 s for sync and prewarm.