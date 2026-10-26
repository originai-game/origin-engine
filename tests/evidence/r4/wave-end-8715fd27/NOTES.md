# Mac wave-end at 8715fd27 — PASS

2026-10-26, macrunner (M4 Pro, macOS 27.2), measure lane
`/Volumes/app/origin-engine-measure`, pinned to
`8715fd279ebb8406e578ade2f91d9e3a4d65adee`. No source, baseline or
reference edits. After fetching and checking out the revision:

```sh
sleep 60
uptime
CI=1 scripts/wave-end.sh --stages gate,web
```

One-minute load at start: 3.08. This is correctness/size validation,
not an idle performance comparison. `CI=1` forces the full gate and fresh
browser references without path-aware reuse. Gate and web ran serially
under the script's locks; other wave stages were not selected.

**PASS: full gate 272 s, fresh web-reference 378 s, total 650 s.**
`summary.txt` and `wave-end.log` are exact copies of the files the script
wrote under `target/evidence/wave-end/`; all individual step logs are
copied from `target/evidence/gate/`.

| Step | Result | Seconds |
|---|---|---:|
| fmt | PASS | 2 |
| clippy | PASS | 2 |
| wasm-build | PASS | 19 |
| test | PASS | 146 |
| wasm-opt | PASS | 160 |
| size-gate | PASS | 36 |
| drift | PASS | 2 |
| bun-test | PASS | 83 |
| determinism | PASS | 23 |
| Fresh web-reference | PASS | 378 |

No test binary/name error, failed frame or frame outside existing
tolerance. No reference was re-recorded. Browser RMSE/outlier values
remain in `web-reference.log`; passing does not imply pixel identity
across rendering backends.

## Handoff regression passes

Exact line from `bun-test.log`, on Bun 1.4.2:

```text
(pass) a handed-off world restores in a new process with a changed layout [708.07ms]
```

The test now uses `process.kill(first.pid, "SIGUSR2")`; no scratch
variant, external kill workaround or increased delay was used for this
wave. This supersedes the failing wave on `2280d17a`.

## The committed lockfile stayed unchanged

`tools/wasm-determinism/Cargo.lock` had Git blob hash
`10902bebcb8680b3a562490826ff75276e26a0e9` before and after the run.
`git diff --exit-code -- tools/wasm-determinism/Cargo.lock` passed.
Unlike the previous wave, no regeneration occurred: these determinism
results used the committed lockfile unchanged. The tracked worktree
was clean after validation and before adding receipts.

## Shipping wasm sizes

Fresh shipping `wasm` profile, brotli-q11. All downloads pass the
unchanged 4,194,304-byte budget (`size-gate.log`):

| Download | Brotli bytes |
|---|---:|
| 3D lanes | 3,230,271 |
| 2D lanes | 2,989,486 |
| 3D publish | 3,202,439 |
| 2D publish | 2,959,060 |

## Determinism

Every executed build agrees (`determinism.log`):

| Digest | Value |
|---|---|
| math_digest | `6117185439544271997` |
| bouncing_boxes_hash | `15551159556835976588` |
| physics3d_hash | `10053948282410406339` |
| physics2d_hash | `3002410719564870113` |
| anim_hash | `12806465523503570878` |
