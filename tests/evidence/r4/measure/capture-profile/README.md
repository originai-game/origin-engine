# Native capture profile: PNG dominates Meridian

Source: the original measurement base `1bb341e5`, release Metal, 1280×720.
Only temporary timing hooks and PNG experiment settings were compiled into
an explicitly selected addon; no engine source changes are committed.
`instrumentation.patch` is the exact experiment, `build.txt` records its
hash, addon hash and build log. Candidate builds were not modified.

One batch, 30 measured captures per scene and setting after 60 warm frames
and three discarded captures; one simulation frame between captures.
The usual full minute below load 8 preceded it. Recorded load during all
eight runs was **5.12–5.59**. No rejected captures or retries.

## Default settings: milliseconds, p50 / p95

| Scene | Whole probe call | Extract + render submission | Readback | PNG encode | Base64 + ABI residual |
|---|---:|---:|---:|---:|---:|
| Game template | 28.244 / 31.135 | 0.307 / 0.466 | 3.576 / 5.020 | 23.829 / 27.241 | 0.445 / 0.526 |
| Meridian Loop | 103.376 / 110.816 | 0.755 / 0.998 | 10.745 / 15.701 | 90.305 / 95.803 | 1.611 / 4.152 |

PNG is about 87% of Meridian's p50 total, and 84% of the template's.
This profiles the earlier approximately 98 ms Meridian result rather than
replacing its acceptance receipt with an instrumented measurement.

The runtime timer measures extraction and applying baked probes. The native
backend separately times render submission and `read_offscreen`, which calls
`capture`: buffer allocation/copy submission, map_async wait, CPU row copy,
swizzle and buffer cleanup. **Readback includes waiting for earlier GPU render
work**, not just PCI/memory transfer; render submission is not GPU execution
time. No GPU timestamps were collected.

PNG includes filtering, level-6 deflate, chunks and CRC. The residual is the
whole synchronous JS probe-call wall time minus those four measured spans.
It includes base64, JSON/ABI conversion and parsing, housekeeping, timer and
two stderr-line overheads. It is not an isolated base64 microbenchmark.
Per-sample residuals are computed before taking percentiles; percentile
columns therefore need not add up to the whole-call percentile.

## Encoding experiments: milliseconds, p50 / p95

| Scene | Setting | PNG encode | Whole probe call | Median PNG bytes |
|---|---|---:|---:|---:|
| Template | level 6, Up (default) | 23.829 / 27.241 | 28.244 / 31.135 | 170,098 |
| Template | level 1, Up | 3.996 / 4.094 | 6.310 / 9.123 | 289,360 |
| Template | level 1, no filter | 3.453 / 3.704 | 7.394 / 8.718 | 245,162 |
| Template | level 0 (stored), no filter | 15.276 / 15.534 | 25.048 / 27.292 | 3,687,768 |
| Meridian | level 6, Up (default) | 90.305 / 95.803 | 103.376 / 110.816 | 832,810 |
| Meridian | level 1, Up | 13.631 / 14.342 | 23.644 / 27.283 | 1,112,405 |
| Meridian | level 1, no filter | 14.347 / 14.951 | 24.207 / 28.255 | 1,204,789 |
| Meridian | level 0 (stored), no filter | 15.094 / 15.520 | 29.384 / 33.198 | 3,687,768 |

Level 1 is miniz's fastest compressed level; level 0 is the uncompressed
ceiling experiment. Stored PNGs still pay filtering/chunk allocation and
CRC over 3.69 MB, then larger base64/ABI work (about 6–7 ms here), so removing
deflate entirely is not the fastest end-to-end option in this implementation.
No-filter did not improve Meridian over level 1 with Up in this batch.
The level-1 result gives a plausible optimization direction, **not a shipped
speedup or proof of a hard 30 ms cap**. Modes were sequential, not randomized;
repeat interleaved trials before choosing among close alternatives.

All PNGs decoded at 1280×720 outside the timed region. The final-frame raw
RGBA SHA-256 is identical across all four settings for each scene
(`pixels.json`); both default frames were visually inspected and show the
expected Start scene / Meridian menu and track. This checks losslessness,
not only that the functions returned. Presentation PNGs may subsequently
be downscaled for repository weight; `pixels.json` describes original pixels.

## Reproduce

From r4/measure with its engine source at the original base:

```sh
git apply tests/evidence/r4/measure/capture-profile/instrumentation.patch
cargo build --release -p og-host-node
cp target/release/libog_host_node.dylib target/r4-profile.node
# Undo only the experiment before the idle-check's source identity check.
git apply -R tests/evidence/r4/measure/capture-profile/instrumentation.patch
bash tests/evidence/r4/measure/idle-check.sh tests/evidence/r4/measure/capture-profile/run bash tests/evidence/r4/measure/capture-profile/run.sh
bun tests/evidence/r4/measure/capture-profile/summarize.mjs
python3 tests/evidence/r4/measure/capture-profile/validate-pixels.py
```

Use a fresh output directory or remove only prior run receipts before rerunning:
the idle monitor appends load samples. `run.sh` explicitly selects the
instrumented addon; it never replaces `packages/engine/native`'s addon.
