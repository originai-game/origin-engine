# Shader cooking per backend: validation in the gate, pipelines cached on Vulkan

## Validation (tests/shaders.rs)

`og_render::shader_sources(full)` lists every WGSL module the renderer
compiles on each tier (34 on the full tier: the lit, physical and shadow
modules for both joint palette modes, and every other module; 26 on the GL
tier). og-render's `tests/shaders.rs` parses and validates each with naga
(all validation flags), resolves its overrides, and writes MSL and SPIR-V
for the full tier and GLSL ES 3.0 (WebGL) for every vertex and fragment
entry of the GL tier, as wgpu does when it makes a pipeline. It runs in
both gate lanes (og-render's tests are in test-quick), takes 0.07 s, and
failed with naga's message and line when a function returning an unknown
identifier was appended to `scatter.wgsl`. Material shaders a game declares
stay validated by the cook (`lit_module_source`).

Nothing is precompiled into the wasm: wgpu takes WGSL on every backend
(the browser compiles WGSL itself; the GL backend translates to GLSL at
pipeline creation), so shipping a second form of each shader would add to
the download for no gain in the browser. "Cooking" is therefore the
validation above plus the native pipeline cache below.

## Pipeline cache (pipeline_cache.rs)

`og_render::set_pipeline_cache_dir(dir)`: a renderer on an adapter with
wgpu's `PIPELINE_CACHE` (Vulkan only in wgpu 30) creates every pipeline
with a `wgpu::PipelineCache` read from `<dir>/<pipeline_cache_key>` and
writes it back after its first frame and when dropped
(`RenderInfo::pipeline_cache`). og-host-node uses `OG_PIPELINE_CACHE`
(default `~/.cache/origin-engine/pipelines`, `off` for none); og-gfx
`OG_GFX_PIPELINE_CACHE`.

First frame (renderer creation, the scene's pipelines and one frame, og-gfx
`setup+first`), three scenes per run:

- **lavapipe** (Vulkan, og-lavapipe image, x86_64 under emulation; three
  interleaved rounds, `pipeline-cache-lavapipe.txt`): showcase 19.3 to 19.8 s
  without the cache, 19.4 to 22.8 s with it; gltf-reference 3.09 to 3.19 s
  and 3.20 to 3.56 s; materials-physical 3.88 to 4.04 s and 3.83 to 4.18 s.
  No gain: llvmpipe's cache data is a 96-byte header (it keeps no compiled
  pipelines), so the second run compiles everything again. A GPU's Vulkan
  driver (Android, Linux, Windows) keeps its pipelines there; no such device
  was available to measure.
- **Metal** (M4 Pro, `pipeline-cache-metal.txt`): wgpu has no pipeline cache
  on Metal, so nothing changed in the code path; the first frame is Metal's
  own shader cache at work: showcase 2.9 s on the first run after a build,
  0.21 to 0.28 s after.
- **Chrome** (WebGPU, WebGL2): no pipeline cache in wgpu's web backends (the
  browser keeps its own); the code path is the same as before.
