# wgpu-hal 30.0.1, patched for the WebGL2 tier

A copy of [wgpu-hal 30.0.1](https://crates.io/crates/wgpu-hal/30.0.1) from
crates.io (`src/`, `build.rs`, the published `Cargo.toml`, licences and
README; examples and the lockfile left out). The root `Cargo.toml`'s
`[patch.crates-io]` puts it in place of the crates.io one, and the
workspace excludes `third_party`, so it is neither formatted nor linted as
ours. The published crate depends on wgpu-types and naga by version, so
patching wgpu-hal alone is enough: `Cargo.lock` differs from the crates.io
build only in wgpu-hal's `source` and `checksum` lines.

The first commit that adds this copy is upstream unchanged (git normalised
one CRLF file, `src/vulkan/descriptor.rs`, to LF). Each patch below is its
own commit after it and touches only `src/gles/`, so the Metal, Vulkan and
DX12 backends are byte for byte 30.0.1. Each reverts alone (`git log --
third_party/wgpu-hal`). Evidence: `tests/evidence/debt3/wgpu-hal-vendor/NOTES.md`.

A second patch, a GLES queue that skipped GL calls setting what was already
set (e71328da), was carried on the same branch and reverted at the merge
(1a1260ba, 2026-10-26): the Mac measured no gain from it on the complex
scene (WebGL2 7.861 → 7.592 ms, less than the run spread), the same reading
as the 2026-10-25 decision in `tests/evidence/r4/perf-webgl2-2/NOTES.md`.
Its test, `tests/web/gl-calls.ts`, went with it.

## Patches

### 1. Timestamp queries on WebGL2 through `EXT_disjoint_timer_query_webgl2`

Applied second (after the copy). Lets og-render's GPU pass timer work on
WebGL2.

Upstream 30.0.1 offers `TIMESTAMP_QUERY` (and its `INSIDE_ENCODERS` and
`INSIDE_PASSES` variants) only for `GL_ARB_timer_query`, which a WebGL
context never lists, so WebGL2 never timed a pass although Chrome has
`EXT_disjoint_timer_query_webgl2`. Its `TIMESTAMP_EXT` counter has 0 bits
in Chrome (ANGLE's Metal backend, D3D11 without `enableTimestampQueries`,
ANGLE GL-EGL on llvmpipe), so `queryCounterEXT` cannot be used; only
`TIME_ELAPSED_EXT` measures. The patch (`adapter.rs`, `queue.rs`,
`device.rs`, `command.rs`, `mod.rs`; also given as a patch on gfx-rs/wgpu
v30.0.1 in `tests/evidence/debt/webgl2-gpu-timing/`):

- `adapter.rs` offers the three features when the extensions list
  `EXT_disjoint_timer_query_webgl2`, and sets the private capability
  `TIMESTAMP_FROM_TIME_ELAPSED` when the context has it and not
  `GL_ARB_timer_query`.
- `queue.rs`, `ElapsedTimestamps`: each timestamp write ends the
  `TIME_ELAPSED` query the previous write of the submit began and begins
  its own; the submit ends the last one. A timestamp is the submit's first
  write (the CPU clock in ns) plus the intervals between, so timestamps of
  one submit differ by exactly the GPU's time. Unwritten, overwritten and
  not yet available queries resolve to 0 (WebGPU's unwritten query), and
  every query does after `GPU_DISJOINT_EXT` reads true. Results are read
  with `get_query_parameter_u64` after `QUERY_RESULT_AVAILABLE` (glow's
  `..._u64_with_offset` panics on the web).
- `device.rs`: a destroyed timestamp query set drops its writes;
  `command.rs`, `mod.rs`: `CopyQueryResults` carries the set's target.

It costs about 1 kB brotli on a 3D module with the GL backend, publish
included, since the choice is made at run time.

### 2. Programs link in the background on WebGL2 (`KHR_parallel_shader_compile`)

Upstream 30.0.1 reads each shader's `COMPILE_STATUS` right after
`compileShader` and the program's `LINK_STATUS` and uniform locations
right after `linkProgram`, so every pipeline waits for its own compile
and link and ANGLE never has two programs at once. On ANGLE's D3D11
backend (Chrome and Edge on Windows) a lit program takes 8 to 11 s there.
The patch (`adapter.rs`, `device.rs`, `command.rs`, `mod.rs`):

- `adapter.rs` sets the private capability `PARALLEL_SHADER_COMPILE` on
  WebGL2 when the context lists `KHR_parallel_shader_compile` (glow
  enables it with the context). Native GL is unchanged.
- `device.rs`: with it, `create_program` starts the compiles and the link
  and reads nothing; the shaders, the names to bind and the immediates
  wait in `PendingLink`. `finish_link` reads the status (and, on a
  failure, each shader's compile log), deletes the shaders, binds the
  blocks and samplers and reads the uniform locations; without the
  capability it runs at once, as upstream did.
- `command.rs`, `mod.rs`: `PipelineInner::linked` finishes a pending link
  the first time an encoder sets the pipeline (wgpu-core encodes a
  command buffer's passes at `finish`, so every pipeline made while it
  was recorded has been linking meanwhile), on the context of the
  encoder's `AdapterShared`. A link that failed panics there, with the
  log, where wgpu's default error handler would have panicked when the
  pipeline was made.

Evidence: `tests/evidence/release/windows-x64-webgl2/COLDSTART.md`. Carry it
over like patch 1, or drop it when upstream defers the link reads itself.

## Moving wgpu to another version

Take that version's crates.io wgpu-hal into this directory as one commit,
unchanged, then carry the patch over as its own commit, or drop it:

- Drop the patch when the new upstream carries an equivalent (the GLES
  adapter offers `TIMESTAMP_QUERY` on WebGL2 and reads it with
  `TIME_ELAPSED`).
- Carry it over when it does not: reapply its commit, resolve by hand
  where it meets upstream's `queue.rs`, and rerun the test that holds it
  (`packages/engine/tests/web/gpu-timing.ts`).
- Drop this copy and the `[patch.crates-io]` line when no patch is left.
