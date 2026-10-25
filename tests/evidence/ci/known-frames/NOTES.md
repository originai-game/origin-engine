# The vulkan-cpu tier's known frame divergences (2026-10-25)

CI on main was red on every run with the same six og-tests failures. Five
were gfx frames that exceed their tolerance on lavapipe against metal's
references, the frame authority (docs/ci.md §CI); one was a behaviour test
that failed only on GitHub's runner. The owner's decision of 2026-10-25:
tolerate the known divergences so that CI is green and a new failure shows.

## Per-scene measurement

Each of the five tests run alone on the orb's lavapipe
(`cargo nextest run -p og-tests --test <binary> <test> --no-capture`,
llvmpipe of Mesa 25.0.7 with LLVM 15.0.6) and read from CI's log of run
40208550455 (96e89601; llvmpipe of Mesa 25.0.7 with LLVM 19.1.7 in the
debian:trixie container). Tolerance for every scene here: RMSE 0.01 and
0.5 % of pixels past 32 levels (`compare.rs` default). 921 600 pixels.

| Test | Scene | Orb RMSE | Orb outliers | CI RMSE | CI outliers | max diff | Listed |
|---|---|---|---|---|---|---|---|
| atmosphere_scenes_match_their_references | atmosphere-fog | 0.00167 | 11 | 0.00166 | 11 | 50 | no (passes) |
| | atmosphere-dusk | 0.00139 | 2 | 0.00138 | 2 | 119 | no (passes) |
| | atmosphere-night | 0.01312 | 558 | 0.01312 | 558 | 234 | yes: RMSE |
| sprites_array_match_their_reference_in_one_draw_per_array | sprites-array | 0.01255 | 434 | 0.01255 | 434 | 216 | yes: RMSE |
| tilemap_lit_matches_its_reference | tilemap-lit | 0.02653 | 17 666 (1.917 %) | 0.02653 | 17 665 | 206 | yes: RMSE and outliers |
| atmosphere_moon_matches_its_reference | atmosphere-moon | 0.01295 | 552 | 0.01295 | 552 | 234 | yes: RMSE |
| water_phase_2_scenes_match_their_references | water-planar | 0.00545 | 337 | 0.00546 | 338 | 108 | no (passes) |
| | water-shadows | 0.00493 | 41 | 0.00493 | 41 | 121 | no (passes) |
| | water-underwater | 0.02253 | 20 797 (2.257 %) | 0.02253 | 20 805 | 105 | yes: RMSE and outliers |

Five scenes are listed in `VULKAN_CPU_KNOWN_DIVERGENCES`
(`crates/og-tests/src/gfx.rs`), each with the larger of the two
measurements. A listed scene passes on vulkan-cpu up to 1.25 times its
listed RMSE and outliers and is reported `TOLERATED`; one within tolerance
is reported for removal; nothing changes on any other tier. Checked on the
orb by editing the list: an unlisted scene (sprites-array renamed) fails as
before, a listed scene whose outliers exceed 1.25 times the entry fails
with "beyond its known vulkan-cpu divergence", and a listed scene within
tolerance (atmosphere-fog) passes with "remove it from
VULKAN_CPU_KNOWN_DIVERGENCES".

## The receive-shadows test on GitHub's runner

`og-tests::shadows surfaces_that_receive_no_shadows_are_lit_as_if_unshadowed`
failed on CI with `sun: 8836 pixels lit as if unshadowed (8836 as with no
wall), 1 elsewhere moved`: one pixel, (529, 273) on the silhouette of the
non-receiving trickster's hat, was (23, 20, 50) with the flags and
(23, 20, 51) with every instance receiving. It is deterministic on a given
machine (two renders of each frame are identical) and depends on the
runner's CPU, because llvmpipe compiles the shaders for the host CPU:

| Run (workflow_dispatch, diagnostics) | Runner CPU | Result |
|---|---|---|
| 40219674253, 40220344805, 40221553235 | AMD EPYC 9V45 | 1 pixel, one blue level |
| 40220844465 | AMD EPYC 7763 | 1 pixel, one blue level |
| 40223255173 | Intel Xeon Platinum 8573C | 0, frames pixel for pixel the orb's |
| orb | Intel Xeon @ 2.60GHz | 0 |

On a failing runner the same pixel is equal in both frames with
`Tonemap::None` and at 4x exposure, so the two shader paths' lit values
differ by less than one 8-bit level, and ACES at exposure 1 rounds them
one blue level apart. Saturating the shadow factor (a guess that the pcf
lerp of four lit texels came out above 1) did not change it (run
40220344805), so the engine is not changed. The test now treats a pixel
within one level on every channel as unmoved, with that reason in the
test; a shader that reads the shadow maps for every instance still fails
it (`sun: 0` pixels gained, checked on the orb by making `look_receives`
return true). It does not belong on the frame list: it is a behaviour
test, not a reference frame.

## What the gate reached once og-tests passed

The gate stops at its first failed step, so with `test` red on every run
CI had never run `bun-test` (packages/engine's tests) on the trixie runner
since it was restored. Once `test` passed, it did:

| Run | Step | Result |
|---|---|---|
| 40224280502 | test | `og-tests::effects a_600_tick_gpu_particle_run_is_deterministic` SIGSEGV: the known intermittent (docs/ci.md §CI, row 81), absent from main's runs 40216031217 and 40218052689 and from every later run here; the five frames TOLERATED, every other test passed |
| 40227110602 | test | 1335 of 1335 passed, the five frames TOLERATED |
| | bun-test | four `og build --publish` tests past their 120 s (asset-stream, web-export x3) and the rendered watched reload (no reload event) |
| 40232473770 | bun-test | those five pass; terrain-stream's two 600-tick runs past their 720 s (they passed in 703.7 s on 40227110602) |

The publish tests take `slow()` (OG_TEST_TIME_SCALE, 3 on CI) as
terrain-stream and sound-chunks already did. The reload test wrote its
change once, 1.5 s after the spawn, before `run.ts` started watching
(it watches only once its renderer is up); it now rewrites the bundle
every half second until the run ends. terrain-stream's limit, a hang
guard, is 360 s (1080 s on CI).

## The green run

https://github.com/originai-game/origin-engine/actions/runs/40237856082
(workflow_dispatch on ci/known-frames at 4e557cc7, the commit before this
note): the `gate` job green. Its steps: fmt, clippy, wasm-build, test
(1335 of 1335 passed, 18 skipped), wasm-opt, size-gate, drift, bun-test
(terrain-stream passed in 732 s, past the old 720 s limit), determinism,
all PASS. The test step's log lists the tolerated frames:

```
known vulkan-cpu frame divergences (.../target/evidence/gfx-known-divergences.txt):
gfx atmosphere-moon: TOLERATED on vulkan-cpu against metal ... RMSE 0.01295 > 0.01000 (rmse=0.01295 max_diff=234 outliers=552)
gfx atmosphere-night: TOLERATED ... RMSE 0.01312 > 0.01000 (rmse=0.01312 max_diff=234 outliers=558)
gfx sprites-array: TOLERATED ... RMSE 0.01255 > 0.01000 (rmse=0.01255 max_diff=216 outliers=434)
gfx tilemap-lit: TOLERATED ... RMSE 0.02653 > 0.01000; 17665 pixels (1.917%) differ by more than 32 > 0.500%
gfx water-underwater: TOLERATED ... RMSE 0.02253 > 0.01000; 20805 pixels (2.257%) differ by more than 32 > 0.500%
```

The run's `perf-nightly` job (workflow_dispatch runs it too) is a
measurement, not part of the gate.
