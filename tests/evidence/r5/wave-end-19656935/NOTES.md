# Fork references and wave-end on Metal

Revision: `19656935ed64f9092e9108b36379ad3a6b9d9b20`, Apple M4 Pro,
Bun 1.4.4. Run on 2026-10-26 in the persistent measure worktree.
No source, tolerance, baseline or existing reference was changed.

## Results

| Stage | Result |
| --- | --- |
| New-field Metal behavior tests | PASS: 19/19 |
| Full gate | PASS: 552 s; all nine steps; 1,383 Rust tests passed, 18 skipped |
| Browser references | FAIL: 487 s; two new WebGL2 scenes below |
| Whole wave | FAIL: 1,039 s |

All existing reference comparisons passed within their tolerances. None was
re-recorded. This does not imply byte-identical output across backends.
WebGPU passed the new references. WebGL2 failures against Metal:

| Scene | RMSE (limit 0.01) | Outliers (limit 0.5%) |
| --- | ---: | ---: |
| water-ripples | 0.01249 | 0.000% |
| hooks-planar | 0.09911 | 4.130% |

The inspected WebGL2 hooks-planar frame shows the fallback magenta card,
not the red mirrored block visible on Metal. Actual frames and diffs are
in `webgl2/`; raw comparison results are in the two JSON files.
No WebGL2 override was recorded to hide either failure.
The two Metal references are also copied into `metal/`. In addition to the
harness's `.diff.png`, `.magick-diff.png` comes from `magick compare
webgl2/<scene>.png metal/<scene>.png webgl2/<scene>.magick-diff.png`.
Both commands exited 1 (different pixels), reporting normalized RMSE
0.0124922 for water-ripples and 0.099113 for hooks-planar.

## Download budget

All four Brotli-q11 downloads PASS the unchanged 4,194,304 B budget.

| Variant | Bytes |
| --- | ---: |
| 3D lanes | 3,243,786 |
| 2D lanes | 2,999,067 |
| 3D publish | 3,217,146 |
| 2D publish | 2,968,478 |

Determinism PASS: every applicable native and wasm build agrees:

| Digest | Value |
| --- | ---: |
| math_digest | 6117185439544271997 |
| bouncing_boxes_hash | 15551159556835976588 |
| physics3d_hash | 10053948282410406339 |
| physics2d_hash | 3002410719564870113 |
| anim_hash | 12806465523503570878 |

The committed lockfiles remained unchanged. The handoff regression passed:
`(pass) a handed-off world restores in a new process with a changed layout [828.22ms]`.

## Recording and visual inspection

Used the existing native og-gfx reference-capture path, which captures the
last of each scene's configured frames before any timing frames. Exact
command and adapter information are in `capture.log` and `capture-report.txt`.
Only the 17 requested scenes were passed to `cargo run -p og-tests --bin
og-gfx -- ... --frames 0 --capture crates/og-tests/references`. The incidental
report.txt overwrite was restored; its output is preserved here instead.

Each PNG was inspected before retaining it:

- water-mesh: narrow blue-green stream beside bare sand.
- water-ripples: textured ripple pattern across the surface.
- water-fresnel: bright sky reflection over dark still water.
- water-glint: isolated bright sun reflection.
- shadow-box: prominent near-post shadow; far post outside the box lit.
- hemisphere-light: pale red floor, blue ceiling, not saturated primaries.
- ao-direct: subtle corner/contact darkening.
- ao-screen-water: flat cyan water, tall post and faint submerged block.
- post-sharpen: crisp block and checker edges; distant checker aliasing visible.
- bloom-unreal: broad colored halos around bright spheres.
- alpha-to-coverage: smooth green circular edge on black.
- hooks-eye-front: yellow left card, magenta right card.
- hooks-planar: red block mirrored on the pale card.
- hooks-screen-ao: red occlusion at the block foot, dark open ground;
  red sampling noise is visible farther away.
- water-planar-two: separate red and blue reflections in the two pools.
- terrain-hidden: magenta background with red scattered posts retained.
- terrain-scatter-unshadowed: posts on pale terrain without post shadows.

Subtle effects were checked with the existing controlled tests, not judged
from thumbnails alone: `cargo test -p og-tests --test tests fork_fields:: --
--nocapture --test-threads=1`. All 19 passed. In particular, water AO kept
0.835/0.883 of foot/open light versus 0.715/0.911 without water; hemisphere
gains were floor (47,0,0), ceiling (0,0,99), metal (0,0,0). These support
retaining the deliberately flat water and desaturated hemisphere references.

After a 60 s pause, ran `CI=1 scripts/wave-end.sh --stages gate,web`.
Start load averages: 3.57/3.61/3.42; end: 5.39/7.50/8.59. Compilation
raised load during the gate; these durations are execution receipts, not
idle performance benchmarks. Full stdout, stage logs and summary are kept
without editing their output.
