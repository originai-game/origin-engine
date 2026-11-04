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
