# P4 cook-anim round: what og-cook, og-anim and og-audio owed

Branch `cook-anim`, Apple M4 Pro, macOS, 2026-10-20, shared with other
streams' gates. Merged with main at 7696db4 (ui-text) before the gate.

| Evidence | What |
|---|---|
| `anim-compress.md`, `anim-compress.json`, `anim-compress-f32.png`, `anim-compress-quantized.png`, `anim-compress.diff.png` | animation sample quantisation and keyframe reduction, sparse morph targets: sizes before/after on every committed animated model, errors, the posed trickster from both cooks |
| `sound-chunks.md`, `chunks.json` | long sounds cooked in chunks: the 3-minute track, resident bytes, sample identity; og run of the idle archetype whole and chunked |
| `textures-ktx2.json`, `textures-array.json`, `textures-toktx.json` | KTX2 orientation and swizzle, texture arrays uploaded as one D2Array, the toktx fallback |
| `wasm-size-after.json`, `check-summary.txt` | the size gate and the path-aware full lane at the branch head |

## Gate

`scripts/check.sh` (path-aware full lane) on the merged tree:
**gate PASSED (9/9 steps, 207 s)**: fmt 4 s, clippy 2 s, wasm-build 3 s,
test 163 s (624 tests, 624 passed, 9 skipped), wasm-opt 0 s, size-gate
116 s, drift 4 s, bun-test 201 s, determinism 66 s. (`--all` is the
coordinator's.)

## Wasm (brotli, per download)

| Variant | Main after ui-text (`P4/ui-text/wasm-size-after-merge-20a6d47.json`) | This branch merged with 7696db4 | Delta |
|---|---:|---:|---:|
| 3D publish | 2,899,979 | 2,913,544 | +13,565 |
| 2D publish | 2,735,444 | 2,749,599 | +14,155 |
| lanes | 3,277,957 | 3,290,490 | +12,533 |

The runtime gained the animation decoder (dequantise and rebuild frames),
the sparse-morph expansion, the chunked sound storage and chunk feeder,
the texture-array loader and og-render's array upload; every encoder
(quantisation, keyframe reduction, chunk cutting, KTX2 fix-up) is in the
cook, native only. 3D publish is 69.5 % of the 4 MiB budget; the budget
is unchanged.

## Frames

No committed reference frame was re-recorded: compression is opt-in, the
lossless encodings (constant channels, sparse morphs) rebuild the same
floats, no template opts into chunked sounds or ships a WAV above the
1 MiB threshold, and no texture changes encoding (the cook's own KTX2
bytes are unchanged; only `KTXwriter`'s cooker version, r3 to r4, moves).
