# Glass over overlay layers: cost

`cargo test -p og-ui --release --test gfx -- --ignored glass_cost
--nocapture`, 2026-10-21, Apple M4 Pro, metal, 1280×720, fine stripes as
the scene, 600 frames per run, three runs per case, best kept; load
average 43 to 48 with a lavapipe container and other agents' gates on
the machine (single runs moved by up to 3 ms: `glass-cost.txt`).

| Case | best ms per frame | against no glass |
|---|---:|---:|
| no glass | 0.414 | |
| four 280×400 panels over the scene (one pyramid, the old path) | 1.886 | +1.47 |
| the four over an overlay bar (the overlay drawn into the glass source, one pyramid after it, a final copy) | 1.419 | +1.01 |
| two layers of two panels, a bar under each (two pyramids) | 4.604 | +4.19 |

The phase 2 figure for the first case was +0.83 ms at load 70 to 90
(tests/evidence/P2/ui/glass-cost.md); under this load the one-pyramid
cases did not separate from each other (the over-content case came out
cheaper), so the measurable cost of the new path is the second pyramid:
each glass layer with overlay content under it adds a pyramid (about a
third of the output's pixels written, six passes). A frame without glass
pays nothing, and one whose glass lies under all of the overlay takes the
old path unchanged. A quieter rerun is owed.
