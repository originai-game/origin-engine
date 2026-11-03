# wgpu 30.0.1, patched for asynchronous render pipelines on WebGPU (fork s60)

A copy of [wgpu 30.0.1](https://crates.io/crates/wgpu/30.0.1) from crates.io
(`src/`, `build.rs`, the published `Cargo.toml`, licences and README). The
root `Cargo.toml`'s `[patch.crates-io]` puts it in place of the crates.io
one; the workspace excludes `third_party`. The first commit adding it is
upstream unchanged; the patch is the next one and touches only
`src/backend/webgpu.rs` (the browser's WebGPU backend) and one re-export in
`src/lib.rs`, so the wgpu-core path (native, WebGL2) is byte for byte 30.0.1.

## The patch

`createRenderPipeline` compiles inside one task of the browser's GPU
process. A frame that makes dozens of pipelines (a large game's first
frame) holds that process, and with it the page's frames and the WebGPU
queue, for as long as they all take (measured in Chrome 155 on SwiftShader:
six large pipelines 2.5–3.2 s with the queue blocked throughout; the same six
with `createRenderPipelineAsync` 1.8 s with the queue answering in 0.25 s,
and a later `createRenderPipeline` of the same content 1–56 ms, Dawn's
content cache).

`wgpu::webgpu::async_pipelines`:

- `set_async(on)`: while on, `Device::create_render_pipeline` asks for
  `createRenderPipelineAsync` and hands the `RenderPipeline` back at once;
  the browser's pipeline fills it in when the promise resolves (`pending()`
  counts the ones not made yet). A refused one stays unmade: its first use
  makes it with `createRenderPipeline`, which reports the error as before.
- `set_skip(on)`: while on, a render pass's `set_pipeline` of a pipeline not
  made yet skips that pipeline's draws (`take_skipped()` counts them)
  instead of making it then. A render bundle never skips (it is replayed),
  nor does `get_bind_group_layout`.

Both are off by default: a caller that never turns them on gets 30.0.1's
behaviour. og-render (`set_async_pipelines`) turns `set_async` on only
around the frame pipelines it makes (`ensure_pipelines`, `Renderer::warm`,
particles), never for one-shot passes (mips, IBL), whose draws must not be
skipped.
