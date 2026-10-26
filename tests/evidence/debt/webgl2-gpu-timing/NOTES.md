# WebGL2 GPU pass timing (debt/webgl2-gpu-timing)

> **Superseded build route (2026-10-26, `debt/wgpu-hal-vendor`).** The wgpu
> fork and the ten-crate git `[patch.crates-io]` below were never merged:
> the same wgpu-hal commit is applied to the vendored crates.io copy in
> `third_party/wgpu-hal` (patch 1 of its ORIGIN.md), and every other wgpu
> crate stays the crates.io one. The fork commands under "Owner action
> needed" are not needed. The root cause, the browser evidence and the size
> below still hold; `tests/evidence/debt3/wgpu-hal-vendor/NOTES.md` has the
> rerun.

The engine's GPU pass timer (`crates/og-render/src/timing.rs`, `GpuTimer`)
now works on the WebGL2 tier wherever the context has
`EXT_disjoint_timer_query_webgl2` (Chrome). It reports through the same probe
members as native and WebGPU: the `gpuTimestamps` capability and
`ProbeTimings.gpuPasses`, plus the trace's gpu thread. No member was added.
Where the extension is missing (Firefox, Safari, Chrome without a GPU on
SwiftShader), it reports `gpuTimestamps=false` and `gpuPasses=[]`, as before.

## Root cause

- wgpu-hal 30.0.1, `src/gles/adapter.rs:530`: `TIMESTAMP_QUERY` (and its
  INSIDE_ENCODERS / INSIDE_PASSES variants) was set only for
  `GL_ARB_timer_query`. The extension list on the web never contains that
  name, so `gpu_timing_absent` said no.
- Turning the feature on was not enough, because every read path was wrong
  on the web:
  - `queue.rs` `C::TimestampQuery` calls `query_counter(TIMESTAMP)`. glow's
    web backend (`web_sys.rs:5940`) forwards this to `queryCounterEXT`, but
    Chrome gives `TIMESTAMP_EXT` **0 counter bits**:
    - ANGLE's Metal backend (`DisplayMtl.mm`: `queryCounterBitsTimestamp = 0`;
      `QueryMtl::queryCounter` is `UNIMPLEMENTED()`).
    - ANGLE's D3D11 backend unless `enableTimestampQueries` is set.
    - The orb's Chrome 156 on ANGLE GL-EGL over llvmpipe, measured below.
    - So a timestamp has no meaning there. Only `TIME_ELAPSED_EXT` measures
      (64 bits everywhere).
  - Results: `get_query_parameter_u64_with_offset` panics on the web
    ("Query buffers are not supported", `web_sys.rs:5979`). The web never
    reaches it, because WebGL has neither `QUERY_BUFFERS` nor `QUERY_64BIT`
    (`full_ver` is `None`). The fallback `get_query_parameter_u32` truncates
    to 32 bits. `get_query_parameter_u64` works (it reads a JS number).
  - WebGL makes a query's result available only in a task after the one
    that ended it, never within it.
  - A `GPU_DISJOINT_EXT` read of true means the GPU's timer was disturbed
    (a power or clock change, a context switch) since the last read. Every
    query in flight is then meaningless.

## The change

Option 1 of the brief: a wgpu-hal patch. The fallback (one `frame` pass
measured by og-host-web) was not needed.

- **wgpu-hal commit `f9b19fb4f4c954708bd7a6e4c6f90c877a10361f`**, one commit
  on top of gfx-rs/wgpu v30.0.1 (`40f4a34`, the source crates.io publishes;
  only the five files below differ). Given as
  [`0001-gles-timestamp-queries-on-webgl2.patch`](0001-gles-timestamp-queries-on-webgl2.patch)
  and as a git bundle, [`wgpu-og-v30.0.1-webgl2-timer.bundle`](wgpu-og-v30.0.1-webgl2-timer.bundle),
  which keeps the commit's hash.
  - `adapter.rs`: offers TIMESTAMP_QUERY, INSIDE_ENCODERS and INSIDE_PASSES
    when the extensions list `EXT_disjoint_timer_query_webgl2`. It sets a
    new private capability, `TIMESTAMP_FROM_TIME_ELAPSED`, when the context
    has that extension and not `GL_ARB_timer_query`.
  - `queue.rs`, `ElapsedTimestamps`:
    - Each timestamp write ends the `TIME_ELAPSED` query begun by the
      previous write in the submission and begins its own, so the queries
      cover the submission back to back. The last one is ended at the end
      of `submit`.
    - A query's timestamp is the submission's first write (taken from the
      CPU clock, `Date.now()` in ns) plus the intervals between. Within one
      submission, which is one frame, the differences are exactly the
      GPU's.
    - Unwritten, overwritten (a generation per query) and not-yet-available
      queries resolve to 0, WebGPU's value for an unwritten query.
    - Every query resolves to 0 when `GPU_DISJOINT_EXT` reads true.
    - Results are read with `get_query_parameter_u64` after
      `QUERY_RESULT_AVAILABLE`.
  - `device.rs`: a destroyed timestamp query set drops its entries.
  - `command.rs`, `mod.rs`: `CopyQueryResults` carries the set's target.
- **Engine.**
  - Root `Cargo.toml` `[patch.crates-io]`: every crate of wgpu's workspace
    that the engine links (naga, naga-types, wgpu, wgpu-core, the three
    wgpu-core-deps, wgpu-hal, wgpu-naga-bridge, wgpu-types) comes from
    `https://github.com/originai-game/wgpu` at `rev = "f9b19fb"`.
    - All of them, not only wgpu-hal: a git wgpu-hal resolves its path
      dependency on wgpu-types inside the git workspace, so wgpu-core and
      wgpu-hal must come from the same source.
    - `Cargo.lock` changes only the `source` lines of those ten packages.
      No dependency was added or moved.
  - `timing.rs`: a frame whose every query resolved to 0 (a disjoint frame)
    is no frame (`frame_of` returns `None`). Before, it was a frame of 0 ms
    passes. The unit test covers this. The module documentation says how
    WebGL2 is timed.
  - `api.ogt`: the `gpuTimestamps` doc string said "never WebGL2". It now
    names WebGL2 with the extension. This is a doc change, not a member;
    `og api gen` rewrote one line in each of index.d.ts, api.schema.json
    and wiki/render.md, and `og api check` is clean.
  - `docs/release.md` row F5: the carried commit must be dropped or moved
    onto the 31 tag when wgpu moves.
  - New test `packages/engine/tests/web/gpu-timing.ts`, in `scripts/check.sh
    web-reference`. Its stand-in extension (`timer-shim.ts`) is shared with
    `tools/bench/gpu-passes.ts`, the per-pass profile below.

**Owner action needed: the fork does not exist yet.** This orb's GitHub
token gets 403 on both `gh repo fork gfx-rs/wgpu --org originai-game` and
`gh repo create`. Until someone runs the commands below, `cargo` cannot
fetch the wgpu crates and the branch does not build. Run once from a
machine with the rights:

```sh
gh repo fork gfx-rs/wgpu --org originai-game --clone=false
git clone --branch v30.0.1 https://github.com/gfx-rs/wgpu.git wgpu-og && cd wgpu-og
git fetch /path/to/origin-engine/tests/evidence/debt/webgl2-gpu-timing/wgpu-og-v30.0.1-webgl2-timer.bundle og/v30.0.1-webgl2-timer:og/v30.0.1-webgl2-timer
git rev-parse og/v30.0.1-webgl2-timer   # f9b19fb4f4c954708bd7a6e4c6f90c877a10361f
git push https://github.com/originai-game/wgpu.git og/v30.0.1-webgl2-timer
```

Until the fork exists, a checkout builds by pointing the URL at a local
clone that has the commit:

```sh
git config --global url."file:///path/to/wgpu-og".insteadOf https://github.com/originai-game/wgpu
CARGO_NET_GIT_FETCH_WITH_CLI=true cargo build ...
```

## Browser evidence (this orb: no GPU, Chrome for Testing 156)

1. **Chrome's real extension.** Chrome with ANGLE GL-EGL over llvmpipe
   (`TRACE_CHROME_FLAGS="--use-angle=gl-egl --enable-unsafe-swiftshader"`)
   exposes `EXT_disjoint_timer_query_webgl2` on the page's own WebGL2
   context, with `TIMESTAMP_EXT` 0 bits and `TIME_ELAPSED_EXT` 64 bits.
   This uses no stand-in.
   - `tools/bench/gpu-passes.ts` on the §7 reference scene (dev build, 1280×720,
     120 frames after 30 warm):
     - Old code: **0 GPU frames, `gpuPasses=[]`**
       ([`gpu-passes-old-real.json`](gpu-passes-old-real.json), 60 frames).
     - Patched: **87 of 120 frames timed**, all 21 render-graph passes, from
       `cascade.0` through `tonemap`
       ([`gpu-passes-real.json`](gpu-passes-real.json)).
   - The numbers are llvmpipe's: about 224 ms a frame, of which main is
     27.9 %, prepass 22.5 % and cascade.2 16.9 %. They are a CPU
     rasterizer's, not a GPU's.
2. **Chrome on SwiftShader, `tests/web/gpu-timing.ts`.** SwiftShader is
   Chrome's default without a GPU, and it does not expose the extension.
   - Without the extension, on both old and new code: `gpuTimestamps=false`,
     `gpuPasses=[]`, CPU passes still timed.
   - With the stand-in (`timer-shim.ts`: ANGLE Metal's shape, so
     TIME_ELAPSED_EXT only, results available a task later):
     - Patched ([`test-new.log`](test-new.log)): `gpuTimestamps=true`.
       `gpuPasses` is `[main, tonemap]`, the frame's `passes` in order.
       Queries are begun and ended in pairs, never nested, never read
       before available, and no `queryCounterEXT` is called.
     - Old code (`--web` of the 6ac9563d dev build,
       [`test-old.log`](test-old.log)): the six checks with the extension
       FAIL (`gpuTimestamps=false`, `gpuPasses=[]`), exit 1.
   - The stand-in also ran the reference scene through `gpu-passes.ts
     --timer-shim` ([`gpu-passes-shim.json`](gpu-passes-shim.json)): the
     same 21 passes, 88 frames.

The timer reads a frame back a frame or two late, and its 8-slot ring skips
a frame while every readback is in flight. That is why some frames are
missing; the rate is the same on WebGPU.

## Size

Shipping 3D lanes module (`wasm` profile, publish, physics3d, shaping,
`scripts/wasm-lanes.sh`, wasm-bindgen, `scripts/wasm-opt.sh --threads`,
brotli -q 11), 6ac9563d against this branch:

| | raw (opt) | brotli |
|---|---|---|
| 6ac9563d | 11,303,092 B | 3,219,380 B |
| this branch | 11,309,885 B | 3,220,476 B |
| delta | +6,793 B | **+1,096 B** (budget 4,194,304 B) |

The shipping module grows although `publish` strips GPU timing. wgpu-hal
decides at run time whether to build timestamps from TIME_ELAPSED queries,
so `ElapsedTimestamps` links into every module that has the GL backend. A
publish build never requests TIMESTAMP_QUERY (`timing.rs`, and og-render's
device request), so that code never runs there. Making it vanish would
need a wgpu-hal cargo feature, which upstream would not take for this.

## Caveat for reading Metal numbers

ANGLE's Metal backend implements TIME_ELAPSED from command-buffer
`GPUStartTime`/`GPUEndTime`. It flushes the Metal command buffer
(`flushCommandBuffer(NoWait)`) at every query's begin and end, so a timed
WebGL2 frame on Metal is cut into one command buffer per pass.
- Per-pass time is what Metal measured for each buffer, and it includes
  that buffer's scheduling.
- The extra commits may also cost a little wall time. Compare the
  `gpu-passes.ts` frame wall with GPU timing on against
  `threejs-compare.ts` (publish, untimed) before calling a pass's share
  exact.

## Mac lane: per-pass numbers on the reference scene, WebGL2

On `/Volumes/app/origin-engine` at this branch's head, once the fork exists
(or with the `insteadOf` above), serially, with the usual idle minute:

```sh
git fetch origin debt/webgl2-gpu-timing && git checkout debt/webgl2-gpu-timing
(cd packages/engine && bun run build)        # scripts/build.sh: dist/web, the dev engine, keeps GPU timers
# 1. The browser check on Chrome with a GPU (real extension; the no-extension run is skipped there):
(cd packages/engine && bun tests/web/gpu-timing.ts)
# 2. Per-pass GPU time of the §7 reference scene, WebGL2 and the WebGPU control
#    (non-publish build of crates/og-tests/reference-scene, 600 frames after 60, trace on):
sleep 60; bun tools/bench/gpu-passes.ts --mode webgl2 --frames 600 --warm 60 --out .amp/in/webgl2-gpu-timing/webgl2.json
sleep 60; bun tools/bench/gpu-passes.ts --mode webgpu --frames 600 --warm 60 --out .amp/in/webgl2-gpu-timing/webgpu.json
# 3. The wall against three.js, as before (publish build, untimed; 600 frames, 60 warm, 2 rounds):
sleep 60; bun tools/bench/threejs-compare.ts
```

`gpu-passes.ts` prints the page's extension and its `TIMESTAMP_EXT` and
`TIME_ELAPSED_EXT` bits, then each pass's mean, p50, p95 and share of the
frame. Its frame (the GPU frame's span) against the WebGL2 wall from
threejs-compare (2.02 ms at 96e89601) answers whether the frame is CPU- or
GPU-bound. It is the same method as the Mac's WebGPU profile of
2026-10-24 (non-publish `webBuild(..., false)`, `frame` calls, `trace
start/take`); this tool keeps it in the repository.

## Upstream text (for gfx-rs/wgpu; nothing was opened)

**Title:** `[gles] Expose TIMESTAMP_QUERY on WebGL2 via EXT_disjoint_timer_query_webgl2`

**Body:**

> The GLES backend only enables `Features::TIMESTAMP_QUERY` when the context
> reports `GL_ARB_timer_query`, so on the web (WebGL2) timestamp queries are
> never offered, even though Chrome exposes `EXT_disjoint_timer_query_webgl2`
> and glow implements it.
>
> Simply enabling the feature is not enough:
> - Chrome reports `QUERY_COUNTER_BITS_EXT = 0` for `TIMESTAMP_EXT`
>   (ANGLE's Metal backend does not implement `queryCounter`, D3D11 gates it
>   behind a feature, and we measured 0 bits on ANGLE GL too), so
>   `queryCounterEXT` yields nothing usable.
> - glow's `get_query_parameter_u64_with_offset` panics on web; WebGL has no
>   `QUERY_64BIT`, so the fallback `get_query_parameter_u32` truncates.
> - WebGL only makes results available in a later task, and
>   `GPU_DISJOINT_EXT` invalidates in-flight results.
>
> This PR builds timestamps from `TIME_ELAPSED_EXT` when the context has
> `EXT_disjoint_timer_query_webgl2` but not `GL_ARB_timer_query` (new
> private capability `TIMESTAMP_FROM_TIME_ELAPSED`):
> - Each `write_timestamp` ends the previous write's `TIME_ELAPSED` query in
>   the same submission and begins its own; `submit` ends the last one.
> - On resolve, a query's value is the submission's first write (taken
>   from the CPU clock in ns) plus the chained intervals, read with
>   `get_query_parameter_u64` once `QUERY_RESULT_AVAILABLE`.
> - Values within a submission differ by exactly the GPU time, which is
>   what `timestamp_writes` users compute.
> - Queries that are unwritten, overwritten, not yet available, or resolved
>   after `GPU_DISJOINT_EXT` read 0, like an unwritten query.
> - `TIMESTAMP_QUERY_INSIDE_ENCODERS`/`_INSIDE_PASSES` are offered too: a
>   write inside a pass just splits the interval.
>
> Tested in Chrome 156 (ANGLE GL-EGL/llvmpipe, which exposes the extension)
> with `timestamp_writes` on every render pass of a ~20-pass
> frame. Without the extension (Firefox, Safari, SwiftShader) nothing
> changes.
