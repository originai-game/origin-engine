# Metal candidate 3: billboard reference and functional lane

Candidate `753e8d9d7b802ae2a68ef75c18222cf7a1a3898d`, Apple M4 Pro Metal,
dev profile, measure worktree. Mac main was not changed. No engine source,
hash expectations, budgets or tolerances changed.

The first functional run failed only because sprites-billboard had no Metal
reference (1,312 Rust tests passed, one failed, 15 skipped); bun-functional
was cancelled when test failed. `initial-functional.log` retains it.

Recorded only the new reference with:

```sh
OG_GFX_CAPTURE=sprites-billboard cargo test -p og-tests --test gfx billboards_face_each_camera_sort_back_to_front_and_cull_by_bounds -- --nocapture
scripts/check.sh functional
OG_ENGINE_PACKAGE="$PWD/packages/engine" packages/engine/native/og.darwin-arm64 test examples/rumpus-isle --filter frames.og.ts
```

Final functional: **PASS**, 111 seconds wall time. fmt 2s, clippy 1s,
test 110s (1,313 passed, 15 skipped), drift reused its earlier 52s PASS,
bun-functional 94s. Package counts: create 16 pass; engine 296 pass,
29 skip; knowledge 49 pass. Cross-host tests retain the functional lane's
stale-wasm skips; this run is not a new wasm validation.

Rumpus: **30 checks pass, 6.960s**, all eleven frames at zero outliers and
RMSE 0.00000. Atlas guard passes. No Rumpus reference changed.

## New billboard reference inspected

The scene contains **eight visible sprites, including two translucent puffs**,
not eight puffs: three figures, two trees, a tilted sign and two puffs.
The ninth sprite is a tree behind the camera. The executed test asserts
8 rendered / 1 culled and byte-identical pixels when the two puffs' submission
order is swapped. The inspected image shows upright trees and figures,
drop shadows, a camera-facing tilted sign, overlapping translucent puffs,
and the side-camera inset. In the inset the sign still faces that camera;
the flat non-billboard figure is nearly edge-on. Pixel-stepped texture edges
are expected for the nearest-filtered artwork and AA Off.

## Only sprites-20k changes against the previous candidate

To separate pre-existing baseline differences from this merge, captured
the gfx scenes at the new candidate and at the previously verified
`15ca814c4637f5e8abf3e859d812448aabcd4695`, using the same dev og-gfx:

```sh
target/debug/og-gfx --all --capture target/r4-metal-3-frames
# In the same worktree temporarily at the previous candidate, then return:
cargo build -p og-tests --bin og-gfx
target/debug/og-gfx --all --capture target/r4-metal-3-control
```

`frames.json` compares new captures with committed references;
`control-vs-candidate.json` compares the two candidates' raw RGBA pixels.
Of 109 shared scenes, **108 are byte-identical**. The only moved scene:

| Scene | Changed pixels | Max channel delta | RGB RMSE | Outliers >32/255 |
|---|---:|---:|---:|---:|
| sprites-20k | 172 | 13/255 | 0.0000571409 | 0 |

The inspected 32× amplified diff contains only sparse pixels, no coherent
structural changes; the after image retains the full sprite field. This is
consistent with the sprite pipeline change and stays within tolerance.
Its existing reference is **not re-recorded**: it already differed from the
previous candidate, and replacing it would also absorb unrelated old drift.
The new capture versus that reference has RMSE 0.0022905, max delta 12/255,
and zero outliers. A rebuilt candidate og-gfx repeat reproduced the new
capture exactly (`sprites-recheck.log`).

Seventeen new captures differ at all from their references, all passing
tolerance. The other sixteen are exactly the previous candidate's output:
fluid-3d, fx-2d, lights-256, materials-array, particles-cpu, particles,
point-shadows-paraboloid, point-spot-lights, sprites-array-fx, terrain-mid,
terrain-near, water-2d, water-3d, water-planar, water-shadows, water-underwater.
They are not new regressions and were not re-recorded. The remaining 93
captures, including the newly recorded billboard, match their references
exactly. Before/after sprite images are 640px presentation copies; the diff
is full-size with channels amplified 32×. Numeric tables use original pixels.
