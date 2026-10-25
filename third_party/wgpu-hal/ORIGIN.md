# wgpu-hal 30.0.1, patched for the WebGL2 tier

A copy of [wgpu-hal 30.0.1](https://crates.io/crates/wgpu-hal/30.0.1) from
crates.io (`src/`, `build.rs`, the published `Cargo.toml`, licences and
README; examples and the lockfile left out). The root `Cargo.toml`'s
`[patch.crates-io]` puts it in place of the crates.io one, and the
workspace excludes `third_party`, so it is neither formatted nor linted as
ours. The first commit that adds it is upstream unchanged; the patch is the
commit after it (`git log -- third_party/wgpu-hal`).

## The patch: the GLES queue skips GL calls that set what is already set

Only `src/gles/` changes: a new `shadow.rs` and `queue.rs` routing its state
setters through it (`mod.rs` declares the module). wgpu-hal's GLES command
encoder assumes the default state at every command buffer and restates a
pass's whole state at every pass and draw (`reset_state` per command buffer,
every attachment detached and attached per pass, `TEXTURE_BASE/MAX_LEVEL`
on every texture bind, vertex attributes per draw). On WebGL every call
crosses from wasm into JavaScript and the browser's validation. In
origin-engine's reference scene that was 3,797 calls a frame; with the
patch it is 1,335, and the frames are unchanged
(tests/evidence/r4/perf-webgl2-2/NOTES.md).

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

Drop the patch (and this copy) when an upstream wgpu-hal carries an
equivalent; moving wgpu to another version means taking that version's
wgpu-hal here and carrying the patch over, or dropping it.
