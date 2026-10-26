# Fork browser reference rerun

Revision: `8a40f5cac48bd620556d4c8d8f3c5e7f5dcb5323`.
Apple M4 Pro, Chrome on ANGLE Metal, 2026-10-26.

After a 60-second pause, ran `CI=1 scripts/wave-end.sh --stages web`.
Result: **PASS web (439s)**. No scene was outside tolerance.
No source or reference edits. This run did not rerun the full gate.

| Scene | Tier | RMSE | Outliers | Reference/result |
| --- | --- | ---: | ---: | --- |
| water-ripples | WebGL2 | 0.00000 | 0.000% | WebGL2 reference; PASS |
| water-ripples | WebGPU | 0.00008 | 0.000% | Metal reference; PASS |
| hooks-planar | WebGL2 | — | — | Not applicable, scenes::Applies |
| hooks-planar | WebGPU | 0.00000 | 0.000% | Metal reference; PASS |

Water-ripples was judged against
`crates/og-tests/references/webgl2/water-ripples.png`. Its informational
comparison against Metal remains RMSE 0.01249; that is not the reference
used for the WebGL2 result. The RMSE limit is 0.01 and outlier limit 0.5%.

The isolated diagnostic's scratch shader and sampler changes were restored
before this checkout; the browser gfx harness rebuilt from this revision.
The tracked source and references remained unchanged throughout this run.

Start load averages: 3.81/3.89/4.62; end: 4.82/5.22/5.19.
Raw console output, wave summary/log, web-reference step log and both
per-scene comparison JSON files are retained beside this note.
