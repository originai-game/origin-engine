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

### 2. The GLES queue skips GL calls that set what is already set

Applied third (after patch 1). Cuts the GL calls a WebGL2 frame sends out
of wasm.

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
