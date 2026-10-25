# Round 4, perf-capture: the probe's capture under its budget

Branch `r4/perf-capture`, from `753e8d9d` (origin/r4/candidate). The
budget (docs/design.md §7): a 1280×720 probe `capture` in at most 30 ms
natively and 60 ms on the web. The Mac lane measured 103.4/110.8 ms
(p50/p95) native and 148.3/150.6 ms web on Meridian Loop, 28.2/31.1 and
49.1/65.1 on the game template, and its profile put 84 to 87 % of the
native call in the PNG encode (zlib level 6) and 1.6 ms in base64 and the
ABI (`tests/evidence/r4/measure/capture-profile/README.md`).

Nothing here ran `scripts/check.sh`, `og test`, web-reference or a
reference capture, and no reference frame was re-recorded. Every number
below is from this orb (16 x64 cores, CPU only, lavapipe for the GPU):
**relative, not the budget's**. The budget is the Mac lane's to measure
at wave end.

## What changed

1. **The encoder** (`og_render::png`, `crates/og-render/src/capture.rs`;
   `og_runtime::png` re-exports it). Rows are still filtered `Up` (the
   first `None`), but each row is deflated as it is filtered by
   **fdeflate** 0.3.7 (image-rs's deflate for filtered image rows: one
   fixed Huffman code for small residuals and runs of zeros, no match
   search) instead of collected into a 3.7 MB buffer and deflated by
   miniz_oxide at level 6. The IDAT is written in place (no copy of the
   compressed stream into a chunk body), and the CRC is slice-by-16 with
   its 16 KB of tables made on first use (the byte-at-a-time CRC was a
   third of the encode once deflate was fast; the tables are not in the
   module). fdeflate was already in Cargo.lock (the `png` crate's fast
   mode, through `image`), not in any wasm module.
   It moved from og-runtime to og-render so og-tests can time and test it
   beside the readback (og-tests does not depend on og-runtime). In the
   `wasm` profile og-render is at opt-level 3 where og-runtime is at s and
   miniz_oxide at z: the old web encoder ran deflate at `-Oz`.
2. **The readback** (`capture.rs`): a capture's staging buffer
   (`COPY_DST | MAP_READ`, 3.7 MB at 1280×720) was created and destroyed
   per capture. A request whose frame was read now hands its buffer back
   to the renderer (one spare), and the next capture of the same size
   copies into it; one cancelled or failed before its mapping landed is
   destroyed as before. The row copy and swizzle are unchanged (fusing the
   swizzle into the copy measured within ±0.3 ms either way, by opt-level).
3. **base64** (`og_probe::base64::encode`, now public): three bytes to
   four characters into a byte buffer instead of a `String::push` per
   character; og-runtime's second copy of the encoder (outputs, saves) is
   gone and uses it. Same text, same save format.
4. **The web Worker** (`packages/engine/src/web/worker.ts`
   `isPendingCapture`): every answer to a `capture` or `sweep` was
   `JSON.parse`d whole to learn whether its result was null; a frame's
   answer is 0.3 to 1.7 MB of base64. It is parsed now only when its text
   holds `"result":null` (every null result does: the probe's JSON is
   compact). 0.77 ms → 0.04 ms for a 1.66 MB answer in Node 26 here.

The probe contract is unchanged: the `png` field is still a base64 PNG,
both hosts take the same code, no IDL change (no `api.ogt` edit, nothing
to regenerate).

## Encoder: what was tried (native, release, p50 of 7, ms / PNG bytes)

Real 1280×720 frames: the committed metal references of the game template
(`intro-8`, the Start scene), fx-showcase (`looks-cinematic-realism`,
`islands`), third-person (`dawn`), match3 (`cascade`) and topdown (`east`).
Every variant's PNG decoded (the `png` crate) to the source pixels.

| Frame | level 6, Up (old) | level 1, Up | **fdeflate, Up, per row (new)** |
|---|---:|---:|---:|
| game intro-8 | 44.9 / 200,353 | 6.9 / 317,976 | **3.3 / 461,002** |
| cinematic-realism | 226.2 / 828,646 | 24.6 / 1,160,498 | **6.6 / 1,245,318** |
| third-person dawn | 301.9 / 967,350 | 29.4 / 1,328,728 | **7.9 / 1,399,301** |
| islands | 245.7 / 790,854 | 23.0 / 1,109,139 | **6.4 / 1,190,087** |
| match3 cascade | 31.6 / 192,473 | 5.4 / 248,053 | **1.7 / 266,749** |
| topdown east (flat) | 9.9 / 29,288 | 1.8 / 65,870 | **1.0 / 245,741** |

(Level 1 here is with the slice-by-16 CRC; with the old CRC it is 0.8 to
2.7 ms slower.) Also measured and rejected, on the same frames:

- miniz level 1 with None or Sub: sometimes faster than Up (intro-8 5.8
  against 6.9 ms, islands 20.1 against 23.5), sometimes slower (wave,
  dawn), always 3 to 4 times fdeflate's time on detailed frames. Avg,
  Paeth and the min-sum-of-abs heuristic over None, Sub, Up and Paeth per
  row: 1.2 to 3.8 times Up's time; Paeth and the heuristic for −9 to +2 %
  of its bytes, Avg for up to 21 % more. Level 2 Up:
  1.2 to 2.1 times level 1's time.
- zlib-rs 0.6.8 (in Cargo.lock, not in any wasm): level 1 Up is 1.6 to
  2.6 times fdeflate's time, for more bytes than either on detailed frames;
  level 2 and RLE slower still. It would also be a new, large wasm dependency.
- fdeflate with Sub, Paeth or None: None is 3.6 to 10 times the bytes,
  Paeth 2.5 to 4 times the time; Sub wins on flat and 2D frames (intro-8
  289 kB in 3.0 ms) and loses on detailed ones. A per-row choice of Up or
  Sub (by sum of abs or by nonzero count) saves 6 to 64 % of the bytes for
  about 3 ms more per frame: more than the base64 and ABI work those bytes
  cost (about 2 ns a byte on the Mac lane: 0.3 ms for the 158 kB it saves
  on topdown), so not taken.
- fdeflate over the whole filtered buffer, or 16 rows at a time, instead
  of per row: the same bytes ±0.01 %, 0.6 to 2 ms slower (the row stays
  in cache).

**Web** (the same code in wasm32 under Node 26's V8, p50 of 7, ms; the
old encoder built at opt-level s, which flatters it: in the shipping
module miniz_oxide is at z; the new one as it ships, the encoder at 3 and
fdeflate at s; making fdeflate opt-level 3 changed nothing, its writer is
generic and compiled in og-render):

| Frame | level 6 (old) | level 1 | **new** |
|---|---:|---:|---:|
| game intro-8 | 104.0 | 12.9 | **8.9** |
| cinematic-realism | 308.9 | 37.7 | **13.5** |
| third-person dawn | 410.6 | 43.8 | **15.1** |
| islands | 329.2 | 35.3 | **12.7** |

A PNG made in wasm decodes (ImageMagick) to the source pixels too.

Expectation for the Mac lane, not a measurement: the native PNG span
(23.8 ms template, 90.3 ms Meridian at p50) should fall by the ratios
above (14 times on the template's frame, about 35 on detailed ones), to
roughly 2 to 3 ms, which leaves Meridian's call at about readback (10.7) +
extraction (0.8) + PNG (≈3) + base64/ABI (≈2.5, more bytes) ≈ 17 ms and
the template's at about 7 ms; the web call loses a larger share, its old
deflate having been at `-Oz`.
Readback's 10.7 ms is mostly waiting for the frame's GPU work, which this
change does not touch.

## End to end in this orb (lavapipe), whole `engine.probe.capture(1280,720)`

`tests/evidence/r4/measure/capture-native.mjs`'s method (60 warm frames,
3 discarded, 30 captures, one frame between; every PNG decoded by
`src/node/png.ts`, whose zlib checks the Adler-32) without its idle
limit (lavapipe alone raises the load past 8), release addons of the
base and of this branch, run alternately:

| Scene | base p50 / p95 ms | this branch p50 / p95 ms | median PNG bytes base → branch |
|---|---:|---:|---:|
| game template, round 1 | 430.5 / 510.7 | 338.4 / 388.6 | 174,796 → 437,219 |
| game template, round 2 | 405.9 / 492.1 | 381.6 / 446.8 | |
| Meridian Loop, round 1 | 5,241.6 / 5,512.7 | 4,033.1 / 4,863.7 | 828,728 → 1,286,792 |
| Meridian Loop, round 2 | 4,596.4 / 4,977.0 | 4,164.0 / 4,551.3 | |

The 30th capture of every run (tick 93) decodes to the same RGBA in all
eight runs, base and branch alike (SHA-256 `bda34267…` for the template,
`95fc93fd…` for Meridian), and the Meridian frame was looked at: the menu
over the neon track and city.

On lavapipe the GPU work is CPU work, so rendering dominates and the
spread is wide; Meridian's gain is larger than the encode alone accounts
for here (noise, or the reused buffer on lavapipe; not separated). This
shows the direction and that every capture decodes, not the budget.

## Wasm size (shipping `wasm` profile, `wasm-opt` as the gate, `brotli -q 11`)

Measured with `scripts/wasm-size-report.sh` as size-2 did, before
(`753e8d9d`) and after:

| Variant | before | after | Δ | share after |
|---|---:|---:|---:|---:|
| 3D publish | 3,322,652 | 3,325,634 | +2,982 | 79.3 % |
| 3D lanes | **3,367,995 (80.3 %)** | 3,370,617 | +2,622 | **80.4 %** |

In the 3D publish module's per-crate table fdeflate is 826 raw bytes of
code, simd-adler32 201, miniz_oxide 6 KB raw smaller (the capture was its
only zlib-wrapped compression call), og-render 3.2 KB raw larger (the
encoder, the CRC, the spare buffer). **The 3D lanes variant is already over
80 % at the base commit** (size-2 left it at 79.2 % from `1bb341e5`; the
candidate's merges since grew it), before this change. The 2D variants
were not measured here (size-2: 72.3 % and 73.2 %).

## Tests

- `og-tests::render::a_capture_png_decodes_to_the_captured_pixels`
  (new, GPU lane): the cooked `.vox` scene (`vox-keyframes`, cooked by
  og-cook as a game loads it) rendered at 1280×720 and at 333×7 (rows
  padded on readback; filtered rows not a whole number of the encoder's
  8-byte words), read back, encoded by `og_render::png` and decoded by the
  `image` crate, which checks every chunk's CRC: the pixels equal the
  capture exactly. Checked to fail with the IDAT CRC off by one bit and
  with the first row filtered `Up` against nothing. On the old code it
  does not compile (`og_render::png` is new); the old encoder was lossless
  too, so ported to it the test would pass: it holds the contract, not
  the speed.
- `og-tests::render::capture_is_bounded_and_cancellable` (extended): a
  capture into the buffer the previous one handed back reads its own
  frame (red after green), and after a cancelled capture took the spare
  the next reads green again. Passes on the old code (no reuse there).
- `og-tests::perf::perf_showcase_capture` (new, `--ignored`, the perf
  lane): the perf harness with each of its 100 frames per run also read
  back and encoded (`perf_of(scene, true)`), against
  `crates/og-tests/perf/<tier>-<adapter>/showcase-capture.json`, and on
  the M4 Pro below 30 ms outright. **No baseline is committed**: there was
  none for capture, and one recorded in this orb would be llvmpipe's. As
  every perf test, it fails until one is recorded on its machine.
  Expected to fail on the old code on the M4 Pro (the old PNG of
  Meridian's frame took 90 ms there); not run on the old code here.
- Ran here, one at a time, on the final tree: `cargo nextest run -p
  og-probe -p og-runtime` (114 passed: the save's base64 round trip, the
  tests that cook PNGs `og_runtime::png` made), `-p og-render` (95
  passed), `-p og-tests --test render` for the three capture tests (3
  passed on lavapipe); `cargo fmt --check`, `cargo clippy --all-targets
  -- -D warnings`, `og api check`, and packages/engine's `bun run
  typecheck` (with `OG_BIN` at a dev `og`).

## Commits

1. `perf(render): capture PNGs deflated by fdeflate as each row is
   filtered` (the encoder, its move to og-render, the two og-tests tests).
2. `perf(render): a read capture's buffer is the next capture's`.
3. `perf(probe): base64 three bytes to four characters at a time`.
4. `perf(web): parse a capture's answer only when its result can be null`.
5. This note.

## What moves

- **PNG bytes**: every `capture`, `sweep` and `screenshot` PNG differs in
  its bytes and is larger: 1.4 to 1.6 times on detailed frames (Meridian
  1.55), 2.3 to 2.5 times on the template's Start scene, up to 8.4 times
  on a flat one
  (topdown 29 kB → 246 kB). The decoded pixels are identical, so no
  reference comparison, frame hash (`og render` hashes RGBA) or
  `expect:frame` moves. Nothing in the repository hashes PNG bytes.
- **Frames, world hashes, the tick**: none. The readback reads the same
  copy into a reused buffer.
- **Base64 text and saves**: identical.

## For the coordinator

1. **3D lanes over 80 % at the base.** 3,367,995 B (80.3 %) at
   `753e8d9d` before this branch, 80.4 % after (+2.6 KB). Something merged
   into the candidate since size-2 grew it; this branch does not fix it.
   Decide whether the capture's +3 KB waits for a size cut or lands now.
2. **Record the capture baseline** on the M4 Pro:
   `OG_PERF_RECORD=1 cargo test -p og-tests --release -- --ignored
   perf_showcase_capture` (and the Chrome tiers if the web perf lane takes
   it), and re-measure the capture rows at wave end
   (`capture-native.mjs`, `web.mjs`) for the budget verdict.
3. **Larger PNGs in git when references are re-recorded.** A reference a
   playtest records is a screenshot's PNG, so the next re-record writes
   files 1.4 to 8 times larger than today's. If repository weight
   matters, the recording path (not the probe) could re-encode at a high
   level; nothing does today.
4. The dev profile (the gate's addon) builds og-render at opt-level 0,
   where the encoder runs; measured here at -O0 it is still 2.5 times
   faster than the old one there (117 vs 294 ms on cinematic-realism, the
   old one having miniz_oxide at opt-level 3 in dev).
5. Left alone: `og run`/`og render`/`og polish` decode a screenshot's
   base64 with `atob` and a per-character loop in JS, outside the timed
   span (file writes). `Buffer.from(data, "base64")` would be faster, but
   `src/node` uses no Buffer anywhere; not changed.
