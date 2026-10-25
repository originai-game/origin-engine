# r4/perf-webgl2: the WebGL2 tier's per-frame accounting

Base: `origin/r4/candidate` @ 753e8d9d. Goal: WebGL2 ≥ 1.5× three.js 0.180.0
instanced on `tools/bench/threejs-compare.ts`'s reference scene (Mac lane:
454 vs 394 FPS, 1.151×).

**Everything below is measured in a CPU-only orb** (Chrome 156 headless,
`--use-angle=gl-egl`, Mesa llvmpipe; SwiftShader refuses the msdf-text shader).
Counts are exact on any GPU; times are relative only and are not acceptance
evidence. The Mac lane re-measures at wave end.

## Where the frame went

Tool: `bun tools/bench/webgl2-trace.ts` (commit 78c242f9) wraps every
`WebGL2RenderingContext` method in the engine Worker and counts calls and
upload bytes per frame (60 warm frames, then 30 counted), or profiles the
Worker (`--profile`, `--gap`).

- Passes per frame (reference scene): ~22. Three shadow cascade layers at
  2048, one 512 spot map, the main MRT pass, AO and post, the bloom chain
  640→20→640, the final pass. 41 draws. **Unchanged by this branch.**
- Uploads per frame: 15 `bufferSubData` (85,904 B) and 1 `texSubImage2D`
  (16,384 B). **Unchanged.** No buffer or texture is re-created per frame,
  and there are no readbacks besides the fence polls of wgpu's queue.
- WebGL calls per frame: **3,797**, of which a model of GL state found
  **2,328 redundant**. The sources are wgpu-hal's GLES backend:
  `texParameteri(BASE/MAX_LEVEL)` on every texture bind (480), the seven
  `disable` calls of `reset_state` at every command-buffer block (452),
  every colour/depth attachment detached and re-attached per pass
  (`framebufferTexture2D` 209), vertex attributes re-specified per draw
  (~840 across enable/disable/divisor/pointer), `bindSampler`/`activeTexture`
  per unit per draw, and 40 `uniform1ui(null, first_instance)` calls.
- Worker CPU per frame (llvmpipe, 3 rounds): busy 135–181 ms, of which
  WebGL native 132–176 ms and wasm ~3.0–3.3 ms. On llvmpipe each GL call is
  far more expensive than on a GPU driver, so the share is exaggerated, but
  the direction holds: WebGL2 pays per call in the browser's validation and
  ANGLE, which WebGPU's command encoding does not.

## What changed

Commit a8454d81, `perf(web): the WebGL2 tier draws through a state filter`:

- `packages/engine/src/web/gl-state.ts`: `filterGlStateOf(canvas)` puts a
  prototype between the Worker's WebGL2 context and
  `WebGL2RenderingContext.prototype` that tracks GL state (caps, masks,
  depth/stencil/blend state, viewport/scissor, program, buffer bindings and
  ranges, VAO attribute state, texture units and samplers, per-texture
  BASE/MAX level, framebuffer attachments) and forwards a call only when it
  changes state. Binds that are only observable at the next draw/clear/
  blit/upload are deferred and flushed there. Any call the filter does not
  model, or a delete of a tracked object, invalidates the matching state, so
  an unknown call can never be skipped wrongly.
- `worker.ts` installs it when the page runs the WebGL2 tier; the WebGPU
  tier is untouched. No Rust, no wasm, no API member changed, so the IDL,
  og-render and og-runtime are untouched and **world hashes cannot move**.
- `docs/web-export.md` bullet, CHANGELOG Unreleased line.

## After (same orb, same scenes)

| | reference base | reference after | spheres (20k) base | spheres after |
|---|---:|---:|---:|---:|
| WebGL calls / frame | 3,797 | **761** (−80%) | 446 | **168.5** (−62%) |
| `texParameteri` | 480 | 0 | 40 | 0 |
| `disable` | 452 | 8 | 72 | 4 |
| `framebufferTexture2D` | 209 | 49 | 20 | 4 |
| `vertexAttribPointer` | 227 | 167 | 8 | 0 |
| `bindSampler` | 287 | 69 | 37 | 32 |
| `bindBuffer` | 348 | 68 | 47 | 13 |
| draws | 41 | 41 | 3 | 3 |
| `bufferSubData` bytes | 85,904 | 85,904 | 2,321,466 | 2,321,466 |

Full per-method breakdown: `counts.json` beside this file.

Worker CPU per frame, reference scene, llvmpipe, `--profile --gap 300`,
60 frames, 3 rounds (CPU-only):

| | busy ms | WebGL native ms | wasm ms | frame p50 ms |
|---|---:|---:|---:|---:|
| base | 135–181 | 132–176 | 3.0–3.3 | 130–172 |
| after | 3.9–5.2 | 0.65–1.0 | 2.5–3.5 | 4.2–5.0 |

The filter's own JavaScript costs ~0.14 ms per reference frame (node bench of
the recorded frame). On a real GPU the base WebGL share is much smaller than
on llvmpipe, so expect a gain well below the orb's 30×; whether it clears
1.5× three instanced is for the Mac lane to say.

## Tests that fail on the old code

`packages/engine/tests/gl-state.test.ts` (bun, 3 tests; there is no
`gl-state.ts` on the base, so all three fail there):

1. The recorded base reference frame
   (`tests/fixtures/webgl2-frame/reference-scene.jsonl`, 3,797 calls) replayed
   through the filter into a GL state model gives the same state at every
   draw, clear, blit and upload as the unfiltered replay, and forwards
   **< 950** calls (the countable contract).
2. 300 random call sequences against the same model (4,000 seeds passed
   once locally): the filtered and unfiltered models agree at every
   observation point.
3. Unknown calls and extension objects invalidate tracked state instead of
   being skipped.

Mutation check: removing any one of the depth-stencil handling, the
range→generic binding update, clear-time caps, the VAO flush, the
unknown-framebuffer handling, stencil face handling, the unit flush, the
FRAMEBUFFER→DRAW_FRAMEBUFFER mapping, the deferred program/framebuffer/
buffer flushes, or the pending range delete makes a test fail.

Existing tests: no Rust changed, so og-render and og-runtime tests are
untouched; `bun scripts/typecheck.ts` and `cargo fmt --check` pass.

## What moves

- **Frames: none should.** The filter only drops calls that leave GL state
  unchanged; the wasm and its codegen are identical. Frame 90 of the
  reference scene on WebGL2, base vs after: `magick compare -metric AE` = 0,
  and the screenshot was inspected as correct. The web-reference `--webgl`
  frames were not run here (AGENTS.md); the coordinator's lane confirms.
- World hashes: unchanged (no simulation code touched).
- Wasm size: unchanged.

## Decisions for the coordinator

1. **Mac re-measure** at wave end: `threejs-compare --mode WebGL2`, reference
   and 20k spheres. This branch cannot prove the 1.5× on a GPU.
2. **Optional: wgpu-core and wgpu-hal at opt-level 3 in the `wasm` profile.**
   Measured here: busy 4.5 → 3.6–4.4 ms, wasm 2.9–3.5 → 2.3–2.8 ms per frame;
   3D publish wasm brotli 3,322,652 → 3,363,912 B (+41 kB, still under the
   4,194,304 B budget). Not adopted; take it if the Mac number is short.
3. What remains per frame after this branch (profile): `og_runtime` extract
   ~2.9 ms inclusive (codec decode allocations), render ~2.1 ms, submit
   ~1.0 ms. The rest of the GL redundancy (`reset_state` per command buffer,
   per-pass attachment churn) comes from wgpu-hal upstream; the filter absorbs
   it, a wgpu patch would remove it at the source.
4. Spheres upload 2.3 MB of instance data per frame on both tiers: a
   candidate for uploading only changed instances, which three does not pay
   for its static instanced mesh. That is the remaining gap on the spheres
   scene (0.694× on the Mac) and needs og-render work, not WebGL-specific.
5. The web-reference `--webgl` frames are the coordinator's to run.
