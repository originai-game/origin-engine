# Bun 1.4.4 Mac wave-end — PASS

2026-10-26, macrunner, M4 Pro, macOS 27.2. Measure lane fetched and reset
to `8b2128b50dc801edc6209cbdbe0e1f7a0b4dc86a`. No source, baseline or
reference edits; receipts only.

## Installation and command

`scripts/install-tools.sh` replaced the existing Bun 1.4.2 at
`/Users/fan/.local/bin/bun`. That was already the active PATH binary;
no PATH edits or alternate installation were needed. Other pinned tools
were already present. The installer log and version output are retained.

```text
/Users/fan/.local/bin/bun
1.4.4
1.4.4+38afee6c8
```

After confirming the version, a 60-second pause preceded:

```sh
CI=1 scripts/wave-end.sh --stages gate,web
```

One-minute load at start: 3.40. `CI=1` forces all steps and fresh browser
references, not a path-aware reuse. Gate and web run serially under the
script's machine/target locks. No complex benchmark was run; no other
wave stages were selected. Timings are gate durations, not a Bun speedup
claim.

## Results

**PASS: gate 321 s, fresh web-reference 398 s, total 719 s.** The wave
summary/log and individual step logs are copied from their original
`target/evidence/wave-end/` and `target/evidence/gate/` locations.

| Step | Result | Seconds |
|---|---|---:|
| fmt | PASS | 2 |
| clippy | PASS | 12 |
| wasm-build | PASS | 31 |
| test | PASS | 167 |
| wasm-opt | PASS | 143 |
| size-gate | PASS | 40 |
| drift | PASS | 5 |
| bun-test | PASS | 88 |
| determinism | PASS | 70 |
| Fresh web-reference | PASS | 398 |

No failed reference frame, frame outside existing tolerance or missing
test binary/name. No reference was re-recorded. Per-frame browser RMSE
and outlier values remain in `web-reference.log`; passing does not mean
pixel-identical results across rendering backends.

Exact handoff test line:

```text
(pass) a handed-off world restores in a new process with a changed layout [698.45ms]
```

The committed `tools/wasm-determinism/Cargo.lock` stayed unchanged, and
the tracked worktree was clean after validation, before adding receipts.

## Shipping Brotli sizes

Shipping `wasm` profile, brotli-q11; all pass the unchanged
4,194,304-byte per-download budget:

| Download | Brotli bytes |
|---|---:|
| 3D lanes | 3,230,271 |
| 2D lanes | 2,989,486 |
| 3D publish | 3,202,439 |
| 2D publish | 2,959,060 |

## Determinism

Every executed build agrees, using the committed lockfile:

| Digest | Value |
|---|---|
| math_digest | `6117185439544271997` |
| bouncing_boxes_hash | `15551159556835976588` |
| physics3d_hash | `10053948282410406339` |
| physics2d_hash | `3002410719564870113` |
| anim_hash | `12806465523503570878` |
