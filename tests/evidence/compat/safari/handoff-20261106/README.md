# Safari and web boot pause handoff

Paused by the owner on 2026-11-06 NZDT. Do not land this WIP as-is.
Source base: `safari` at 52234e917 in `/Volumes/app/origin-engine-lane-12`.
Thread: https://ampcode.com/threads/T-01a1ab98-f3aa-7748-9669-1ffd19538764.

The active validation chain and pipeline series were stopped. No run from
this thread remains, and its GPU slots, exclusive lock, and waiters were
released. Other streams' GPU locks were left alone. The Windows thread
also stopped its requested boot comparison and released its locks.

## Edge differences and automatic quality

The diagnostic patch is **applied in this WIP**, with `diagHistory` added.
Do not apply `probes/diag-auto-quality.patch` again. Remove the temporary
frame-answer fields and reference.ts logging before landing a fix.

The first diagnostic run used a fresh full `run.sh build` from 52234e917.
Automatic quality changed scale during the manual meridian-loop run:
`[tick, scale] = [1,1], [461,.9], [611,1], [1163,.9], [1326,1],
[4979,.9], [5129,1]`. See `diag-meridian.log`.

That is a defect, but it does not explain the title: the title already
failed at tick 72, before any scale change. A title-only control now fails
identically in Safari and Chrome: 2.441% outliers and RMSE 0.04132 against
Metal. Both report scale 1 and history `[frameIndex:73, taaHistory:73]`.
The earlier title-alone pass no longer reproduces on this freshly built
wasm. Do not assume that the old dist matched the rebased source.
The browser-wide fixes thread T-01a1ab00-954d-7597-9ede-a98150bd6e80 was
sent this shared-browser finding. No references or tolerances changed.

The candidate fix adds a host-only `set_probe_frame` bracket in the runtime
and `og_web_probe_frame` in wasm. The Worker's manual `frame` call uses it
with `finally`. Probe frames use configured scale with no adaptive preset
cap, do not train automatic quality, and preserve the live controller for
the next display frame. The game API and IDL are unchanged.

Validation completed:

- The regression test failed before the behavior change: probe frames
  trained quality down to scale .5, preset step 2, frameMs 100.
- `cargo test -p og-runtime --test tests display::`: 10 passed afterward.
- `bun run typecheck` in packages/engine passed.
- The dev wasm rebuilt successfully with the candidate fix.
- Safari's complete meridian-loop run stays at scale 1 through tick 7272.
  It still fails all ten Metal comparisons. Title is unchanged; ghost and
  pause move slightly. This is not a claim that the visual discrepancy is
  fixed. See `fixed-meridian-safari.log` and its JSON.
- Chrome's full fixed run was stopped while queued. Neither fixed looks
  run started. No full gate, clippy, or release validation was run.

Next, in a fresh continuation after the pause is lifted, rebuild the dev
wasm if the source changed:

```sh
cargo build --profile wasm --target wasm32-unknown-unknown -p og-host-web --lib
wasm-bindgen --target web --out-dir packages/engine/dist/web --out-name origin-engine target/wasm32-unknown-unknown/wasm/og_host_web.wasm
```

Then run the controls separately. Each run.sh invocation takes one slot:

```sh
sh tests/evidence/compat/safari/run.sh example --chrome meridian-loop
sh tests/evidence/compat/safari/run.sh looks
sh tests/evidence/compat/safari/run.sh looks --chrome
```

Compare Safari directly with Chrome, particularly bold-comic-illustration,
illuminated-manuscript, voxel-worlds, and sculpted-clay. Distinguish shared
Metal differences from Safari-only failures. Inspect any differing frames.
The published wasm variants predate the new host export: rebuild those
before testing a new Worker against a publish module.

## Safari fx pipelines: no new measurements

The inherited 17.3 s Safari versus 3.2 s Chrome fx run is still the only
reported comparison. No new pipeline-series run obtained every GPU slot
before the pause, so there are **zero new pipeline timing results**.

The harness from lane-1 commit c270745dd is copied into
`../pipelines/`. `series.sh` now derives its checkout, does not wait for an
unrelated wasm.log, and takes all slots once per timing run. `measure.ts`
now fails on a stalled clock or incomplete station, records actual ticks,
and preserves pipeline logs. Shell syntax checked; exports built for sync,
async, and prewarm. The end-to-end harness itself is not yet validated.

Next: build the matching wasm, then run
`bash tests/evidence/compat/safari/pipelines/series.sh`.
It compares Safari and Chrome, sync/async/prewarm, early/late tap, three
repetitions, with a separate exclusive acquisition per run. Keep Chrome on
Mac sync unless new evidence contradicts the existing cold-boot table.

Source clue, not proof about the installed binary: WebKit revision
d953bbedb (Safari's 7625.2 branch) calls synchronous `createRenderPipeline`
inside `createRenderPipelineAsync` and only schedules its callback. Its
`renderPipelineState()` calls Metal's synchronous constructor lazily.
Current WebKit main has a different async implementation. Check the actual
Safari version and measured behavior before choosing a policy.
Source: https://github.com/WebKit/WebKit/blob/d953bbedb/Source/WebGPU/WebGPU/RenderPipeline.mm#L1816-L1825

## Firefox first tap: background warm-up implicated, prewarm not a fix

Windows thread: T-01a1ab9d-3662-750e-b906-7429d51e0ebd. Its raw results and
scratch instrumentation are copied here. They are diagnostic evidence,
not shared engine changes. Engine/wasm revision: 144f9b19c. The initial
runner checkout was a156e7dae; later scratch runs used checkout 54b532622
without rebuilding the wasm. No cold driver-cache claim is established.

The old diagnostic inserted a polling loop before the original 600 ms
assertion. Its apparent 2.2 s pass was invalid. The replacement leaves the
tap, 600 ms wait, and tick assertion intact, then collects extra evidence.
Times below are pointerdown to a completed frame exposing tick advancement,
not timestamps inside Rust's simulation tick.

- Chrome's four dev modes pass at 39–56 ms.
- Firefox sync isolated and nonisolated initially fail at tick 1→1.
  Play reaches the Worker in 35/15 ms, but the first rAF callback begins
  about 1496 ms after pointerdown. Advancement is observed around
  1531/1507 ms. `engine.frame` takes 2–5 ms. There are no pipeline-creation
  API calls after the tap. Firefox async passes at 126/111 ms.
- Warm-up enabled control: isolated fails, advancing at 1592 ms.
  Three warm pipelines were issued 14.30, 9.78, and 5.22 ms before tap.
  All six queue-before/after callbacks resolve at 1542.34–1542.36 ms.
  Nonisolated passes at 72 ms.
- Disable only `warmBehindGate`: both modes pass at 92/73 ms. This
  implicates outstanding warm-up work, but does not distinguish backend
  compilation from GPU-service or callback scheduling delays.
- A subsequent sync/prewarm comparison was interrupted after ten modes.
  Sync isolated advancement: 1539.4/1176.76/170.64 ms; nonisolated:
  103/534/81 ms. Prewarm isolated: 1666.4/3777.7 ms; nonisolated:
  1580/3747 ms. Third prewarm pair did not finish. First-frame boot.at:
  sync 1318–1440 ms, prewarm 1257–1341 ms; first frames all 9 sync,
  0 async, 0 fallback. **Do not switch Firefox auto to prewarm:** completed
  evidence does not support it. See `firefox-sync-prewarm.log`.

Next: inspect the preserved queue/async-resolution events, then measure
the GPU-service work around the delayed callback on studio-win. Preserve
the original 600 ms assertion and first-frame draw. Do not replace the
problem with a timer-driven tick or widen the window. No pipeline policy
was changed in this WIP.

## Delivery and disposable outputs

Earlier evidence landed before this pause: d3ff3a1f9 as ebaae82d8 and
52234e917 as 902a5eed7. This WIP is separate and must not go to main or
the art runner during the pause.

Lane-1's harness was copied before its authorized reset to origin/main and
`lane.sh free lane-1`. Its untracked brasswick and rumpus skin directories
were preserved. They may prevent the lane's clean-idle check from passing.
The main checkout was never switched or reset.

All lane-12 `packages/engine/dist/compat/*` directories are disposable
render outputs, not reference sources. This continuation wrote
`safari-example-meridian-loop`, `safari-fx` (title-only), and `chrome-fx`
(title-only). The newer Chrome full run never started, so its existing
`chrome-example-meridian-loop` is inherited, not a fixed-run result.
Selected JSON and logs are committed here. Title images are also in
`.amp/in/safari/title-before.png` and `title-alone.png`. No dist directory
was deleted at the pause. Keep target/ and the lane worktree.

The three `.amp/in/sp/dist-*` pipeline exports can be regenerated. Logs
in `.amp/in/safari/` and the Windows uploads in the main checkout's
`.amp/in/scratch/` are scratch copies of the evidence preserved here.
