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
one CRLF file, `src/vulkan/descriptor.rs`, to LF). Each patch below is one
commit after it, in the order listed, and touches only `src/gles/`, so the
Metal, Vulkan and DX12 backends are byte for byte 30.0.1. Either patch can
be reverted alone (`git log -- third_party/wgpu-hal`). Evidence for both:
`tests/evidence/debt3/wgpu-hal-vendor/NOTES.md`.

## Patches, in the order applied

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

### 2. The GLES queue skips GL calls that set what is already set

Applied third (after patch 1). Cuts the GL calls a WebGL2 frame sends out
of wasm.

Only `src/gles/` changes: a new `shadow.rs` and `queue.rs` routing its state
setters through it (`mod.rs` declares the module). wgpu-hal's GLES command
encoder assumes the default state at every command buffer and restates a
pass's whole state at every pass and draw (`reset_state` per command buffer,
every attachment detached and attached per pass, `TEXTURE_BASE/MAX_LEVEL`
on every texture bind, vertex attributes per draw). On WebGL every call
crosses from wasm into JavaScript and the browser's validation. In
origin-engine's reference scene that was 3,797 calls a frame; with the
patch it is 1,335, and the frames are unchanged
(`tests/evidence/r4/perf-webgl2-2/NOTES.md`, held by
`packages/engine/tests/web/gl-calls.ts`).

`Shadow` holds the state one `Queue::submit` has set and skips a setter
whose value it saw set within the submit:

- It starts unknown at each submit (anything may touch the context between
  submits) and forgets everything after a command it does not follow
  (copies, clears of buffers, barriers, query copies: `Shadow::follows`),
  keeping only the per-texture base and max level, which only `BindTexture`
  sets.
- The draw framebuffer's attachments, the capabilities (depth, stencil,
  scissor, blend, cull, polygon offset, alpha to coverage, depth clamp) and
  the attribute arrays' enables are deferred to the next command that
  observes state (`Shadow::sets_only` lists the setters), then applied only
  where they differ from what GL has.
- The active texture unit and the `ARRAY_BUFFER` binding are set only when
  a texture bind or an attribute pointer needs them.
- Everything set some other way (per-draw-buffer masks and blend, the GLES
  3.1 separate attribute format, multiview and multisampled attachments,
  external framebuffers) makes the matching state unknown.

Where it meets patch 1: a `TimestampQuery` is neither a setter nor a
command the shadow forgets after, so what is pending is applied before the
query's `TIME_ELAPSED` interval is begun and ended (which sets no state the
shadow keeps), and `ElapsedTimestamps::end_submission` runs after the last
command buffer's state is applied.

## Moving wgpu to another version

Take that version's crates.io wgpu-hal into this directory as one commit,
unchanged, then carry each patch over as its own commit in the order above,
or drop it:

- Drop a patch when the new upstream carries an equivalent (patch 1: the
  GLES adapter offers `TIMESTAMP_QUERY` on WebGL2 and reads it with
  `TIME_ELAPSED`; patch 2: the GLES queue stops restating unchanged state).
- Carry a patch over when it does not: reapply its commit, resolve by hand
  where both meet in `queue.rs`, and rerun the tests that hold it
  (`packages/engine/tests/web/gpu-timing.ts` for patch 1,
  `packages/engine/tests/web/gl-calls.ts` for patch 2).
- Drop this copy and the `[patch.crates-io]` line when no patch is left.
