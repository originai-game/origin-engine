# debt3/wgpu-hal-vendor: one carried wgpu-hal, two GLES patches

Branch `debt/wgpu-hal-vendor` from `origin/main` at db10aaf9. It replaces
two unmerged branches that each patched wgpu-hal 30.0.1 in a different way:

- `debt/webgl2-gpu-timing` (e72891d3): GPU pass timing on WebGL2. It took
  ten wgpu crates from a git fork, `originai-game/wgpu` at f9b19fb. That
  fork was never created.
- `r4/perf-webgl2-2` (7ebac12e; `r4/hal-shadow` 049d7937 has the same two
  commits): the vendored copy and the GLES queue shadow.

Evidence of the old branches, kept where they put it:
[`tests/evidence/debt/webgl2-gpu-timing/NOTES.md`](../../debt/webgl2-gpu-timing/NOTES.md)
(root cause, browser evidence, size, the upstream text; brought over by
this branch with a note that the fork route is superseded) and
[`tests/evidence/r4/perf-webgl2-2/NOTES.md`](../../r4/perf-webgl2-2/NOTES.md)
(the shadow's counts and frames, and the coordinator's decision of
2026-10-25; already on main, with `counts.json` beside it).

All numbers below were measured in a CPU-only orb: Chrome for Testing 156,
headless, with `--use-angle=gl-egl` on Mesa llvmpipe, 16 cores. Call
counts are exact on any GPU. No timing here is acceptance evidence.

## What the branch does, commit by commit

```
db10aaf9  main
79471987  build: carry wgpu-hal 30.0.1 in third_party, unchanged       (c4e0f4c7, rebased)
a11fe16e  wgpu-hal (gles): timestamp queries on WebGL2 ...             (patch 1 = f9b19fb)
e71328da  wgpu-hal (gles): the queue skips GL calls that set ...      (patch 2 = c5f57381)
6a1aaea4  og-render: GPU pass timing on WebGL2 ...                    (4f065859 minus Cargo)
70208cac  tests/web/gpu-timing.ts, tools/bench/gpu-passes.ts ...      (e72891d3)
2d806ed7  tests/web/gl-calls.ts: ... fewer than 1,500 GL calls        (new)
          docs and this evidence
```

- **The copy.** `third_party/wgpu-hal` is crates.io wgpu-hal 30.0.1
  byte for byte. `diff -r --strip-trailing-cr` against
  `~/.cargo/registry/src/*/wgpu-hal-30.0.1` shows only `ORIGIN.md` added
  and the examples, `Cargo.lock`, `Cargo.toml.orig` and cargo's stamps left
  out. The root `[patch.crates-io]` gains `wgpu-hal = { path =
  "third_party/wgpu-hal" }` beside `basis-universal-sys`. The published
  crate depends on wgpu-types and naga by version, so patching wgpu-hal
  alone builds. Against main, `Cargo.lock` loses only wgpu-hal's `source`
  and `checksum` lines. No dependency was added or moved, and wgpu,
  wgpu-core, wgpu-types and naga stay the crates.io ones.
- **Patch 1 (timer)** is `0001-gles-timestamp-queries-on-webgl2.patch`
  applied with `git apply -p2 --directory=third_party/wgpu-hal`. It applied
  cleanly, with no hunk edited. Files: `src/gles/{adapter,queue,device,command,mod}.rs`.
- **Patch 2 (shadow)** is c5f57381 cherry-picked on top. Both patches
  touch `queue.rs`. Git merged them without conflict, and I checked by hand
  where they meet:
  - `process()` gained the shadow's `&mut Shadow` parameter. The timer's
    `TimestampQuery` and `CopyQueryResults` arms only add code inside their
    own arms.
  - A `TimestampQuery` is not in `Shadow::sets_only`, so the pending
    attachments, capabilities and attribute enables are applied before the
    `TIME_ELAPSED` interval ends and the next begins. The pass's state
    changes therefore fall in the interval they belong to.
  - `TimestampQuery` is in `Shadow::follows`, so the shadow keeps its state
    across it. `begin_query` and `end_query` set nothing the shadow
    tracks.
  - `CopyQueryResults` is not followed. The timer's resolve only reads
    query parameters, and after the copy binds its buffer the shadow
    forgets, as it did before.
  - `ElapsedTimestamps::end_submission` runs after the loop over command
    buffers, so after the last `shadow.flush`.
- **Either patch reverts alone:** `git revert --no-commit a11fe16e` and
  `git revert --no-commit e71328da` each apply cleanly on the head.
  `ORIGIN.md` is laid out so that each patch commit fills only its own
  section. ORIGIN.md describes both patches, their order and the rule for
  a wgpu move.
- **Engine side**, from debt/webgl2-gpu-timing as it was:
  - `timing.rs`: a frame whose every query resolved to 0 is no frame.
  - The `gpuTimestamps` doc line, with `og api gen`'s three outputs.
  - `tests/web/gpu-timing.ts` with its page, worker and `timer-shim.ts`,
    and its line in check.sh's web-reference step.
  - `tools/bench/gpu-passes.ts` and the evidence directory.
  - The module doc of `timing.rs` and release.md F5 now name the vendored
    copy, not the fork.
- **New test**, `packages/engine/tests/web/gl-calls.ts`, runs in
  web-reference beside `lane-panic.ts`. Below.

## Tests, and how each fails on the old code

| Test | Branch | Old code |
|---|---|---|
| `tests/web/gl-calls.ts` (new) | **pass**: 1,335 calls a frame leave wasm, 732 reach WebGL ([`gl-calls-branch.json`](gl-calls-branch.json)) | **FAIL** with the shadow commit reverted: 3,793 / 761 ([`gl-calls-shadow-reverted.log`](gl-calls-shadow-reverted.log)), and on main db10aaf9: 3,793 / 761 ([`gl-calls-main.log`](gl-calls-main.log)) |
| `tests/web/gpu-timing.ts` | **pass**, both in Chrome's default SwiftShader (with the stand-in extension, then without it: 11/11, [`gpu-timing-swiftshader.log`](gpu-timing-swiftshader.log)) and on ANGLE GL-EGL with Chrome's own extension (5/5, the no-extension run skipped as designed, [`gpu-timing-gl-egl.log`](gpu-timing-gl-egl.log)) | **FAIL** on the unpatched hal: the six checks with the extension (`gpuTimestamps=false`, `gpuPasses=[]`; the old branch's [`test-old.log`](../../debt/webgl2-gpu-timing/test-old.log) on 6ac9563d; the hal code is the same 30.0.1) |
| og-render `timing::tests::passes_run_back_to_back_...` (extended) | pass (88/88 in og-render's lib) | a frame of all-zero queries returned a frame of 0 ms passes, not `None` |

`gl-calls.ts` counts the way `tools/bench/webgl2-trace.ts` does, in the
engine Worker before its first line. It also counts in front of the JS
state filter (`src/web/gl-state.ts`):

- The filter puts its methods on a prototype between the context and
  `WebGL2RenderingContext.prototype`. When counting starts, the test puts
  one more prototype in front of the filter's, which counts every method.
- `WebGL2RenderingContext.prototype` itself is wrapped too, as
  webgl2-trace.ts wraps it. That gives the count behind the filter, the
  calls that reach WebGL.
- The checks:
  - the page runs the WebGL2 tier;
  - the context draws through the filter;
  - the counter sees the scene's 41 draws a frame, so a counter that sees
    nothing cannot pass;
  - fewer than 1,500 calls a frame in front of the filter;
  - no more calls behind the filter than in front of it.

Behind the filter, the patched and unpatched hal differ by 4 % (732 vs
761), too little to hold the patch. In front of it they differ by 2.8×.
The test builds the reference scene's publish web half, which needs
`dist/web-publish` from `scripts/build.sh`, and serves it without
isolation. It drives 60 frames after 60 warm frames.

**Running it without a GPU.** Chrome 156's SwiftShader refuses one of the
reference scene's shaders on WebGL2, so in an orb run it with
`TRACE_CHROME_FLAGS="--use-angle=gl-egl"`, the variable webgl2-trace.ts
reads. `gpu-timing.ts` takes no flags, so to give it the real extension on
llvmpipe I passed `--chrome` a wrapper script that appends
`--use-angle=gl-egl`. On the Mac both run with default flags.

The GPU timer on the merged hal, on the reference scene with Chrome's real
extension (`TRACE_CHROME_FLAGS=--use-angle=gl-egl bun
tools/bench/gpu-passes.ts --mode webgl2 --frames 60 --warm 20`):

- "extension true (TIMESTAMP_EXT 0 bits, TIME_ELAPSED_EXT 64)";
- 29 GPU frames of 60, all 21 render-graph passes, cascade.0 to tonemap
  ([`gpu-passes-gl-egl.txt`](gpu-passes-gl-egl.txt)).
- The times are llvmpipe's, about 265 ms a frame.

The shadow's deferral leaves the timer's intervals intact.

## The frame

Reference scene, WebGL2 tier, frame 90:

- Command: `TRACE_CHROME_FLAGS=--use-angle=gl-egl bun
  tools/bench/webgl2-trace.ts --only reference --warm 90 --shot`, each
  build in its own `--dist`.
- Branch against main db10aaf9: `magick compare -metric AE` **0**.
- Branch against the branch with the shadow reverted: **0**.
- The frame was inspected: the prop grid of white and olive cubes, red and
  metallic spheres, the walkers with their shadows on the ground, sky and
  horizon.
- Native tiers are unaffected: Metal, Vulkan and DX12 code in the copy is
  byte-identical to 30.0.1, and both patches are under `src/gles/`.

## Builds and checks run here, one at a time

- `cargo check -p og-render -p og-runtime -p og-host-web`: native and
  `--target wasm32-unknown-unknown`, 0 warnings.
- `cargo clippy` of the same crates, `-D warnings`: native and wasm32,
  pass.
- og-render on wasm32 with `--features profile2d`, `publish`, and
  `profile2d,publish`: `cargo check` and `cargo clippy -D warnings` pass
  for each.
- `cargo fmt --check`: clean. `third_party` is excluded from the
  workspace, so the copy is not formatted as ours.
- `cargo test -p og-render --lib`: 88 passed.
- `og api check`: "the generated files match crates/og-api/api.ogt".
- `shellcheck -x scripts/check.sh`: clean.
- web-reference's path rule (`rule_web`, from `cargo tree` of
  og-host-web) lists `third_party/wgpu-hal`, so a change to the copy
  reruns the browser tier.

Not run here (AGENTS.md): the gate, `og test`, web-reference, reference
captures, determinism.

## Wasm size

3D publish module, single-threaded (`wasm` profile, features
`publish,physics3d,shaping`, `scripts/wasm-opt.sh`, `brotli -q 11`), all
three built here from the same target dir:

| Build | Brotli | Delta |
|---|---:|---:|
| main db10aaf9 | 3,200,441 B | |
| copy + patch 1 (shadow reverted) | 3,202,118 B | +1,677 B |
| branch (copy + both patches) | 3,205,199 B | +3,081 B more, **+4,758 B** total |

Expect about **+4.8 kB brotli** on the 3D publish module, and about the
same on the 3D lanes module. The old branches' own measurements were
+1,096 B for the timer (3D lanes) and +5.2 kB for the shadow (3D publish),
so up to about +6.3 kB if the coordinator's size gate agrees with them.

- The timer code links into every module with the GL backend, publish
  included, though a publish build never requests TIMESTAMP_QUERY.
- The 2D modules link the same hal code: expect a similar delta there.
- The size gate measures it at the wave end.

## What the Mac lane owes

On this branch's head, serially, with an idle minute between runs:

1. `scripts/check.sh web-reference`. It now includes `gl-calls.ts` (no
   flags needed with a GPU) and `gpu-timing.ts`. Every WebGL2 reference
   frame should be unmoved: AE 0 here on the reference scene, and the GL
   calls that reach WebGL are the same in kind.
2. `cd packages/engine && bun tests/web/gpu-timing.ts` on Chrome over
   Metal: Chrome's own extension, with the no-extension run skipped.
3. `bun tools/bench/threejs-compare.ts --only complex` for the WebGL2
   number. The complex scene judges the 1.5× (row 51).
4. Optionally, `bun tools/bench/gpu-passes.ts --mode webgl2 --frames 600
   --warm 60` for per-pass Metal times. The debt/webgl2-gpu-timing NOTES
   explain how to read them: ANGLE Metal splits command buffers at each
   query.

## For the coordinator to decide

- **The shadow (patch 2) reverses the decision of 2026-10-25.** That
  decision is recorded in `tests/evidence/r4/perf-webgl2-2/NOTES.md`:
  - The vendored hal and shadow were not adopted because the Mac measured
    no gain from them on `threejs-compare --mode WebGL2`:
    - reference scene: 1.952 → 1.993 ms;
    - spheres: 3.742 → 3.901 ms;
    - both within noise.
  - This branch carries it because the brief asks for it. The copy is now
    needed for patch 1 anyway, so the shadow's own cost is about 450 lines
    to carry over and +3.1 kB brotli.
  - If item 3 above shows no WebGL2 gain on the complex scene, dropping
    the shadow is one clean `git revert e71328da`, with `gl-calls.ts` and
    its check.sh line removed alongside.
- No phase-1 row named these branches. Rows 100 and 101 in
  `docs/phase-1.md` are new and may collide with another stream's numbers
  at the merge.
