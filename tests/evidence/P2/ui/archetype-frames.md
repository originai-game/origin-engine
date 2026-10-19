# Phase 2, UI round: the archetypes after the round

On main at b2532cd, `og test` failed on four archetypes the UI round never
re-recorded: the `frames` suite of idle, rhythm, roguelike and tower-defense,
and tower-defense's lose, pad, pause and replay, whose first tap on the
title's Play came too early.

## The title that ignored early taps

Cause: og-ui's `apply_stagger` (crates/og-ui/src/ui.rs) marked every
staggered control inert until its own place in the cascade had played. Before
the round this never mattered for skinned buttons. The old eased hover
answered a tiny nonzero lift at rest, so each skinned button carried an
effect, and the stagger skips any node that already has one. The round's
springs (814fdad) rest at exactly zero, so the buttons joined the cascade as
og-ui's README describes, and with it the inertness. Pathguard's title
(`stagger 50`, `durations base 280`) then refused Play until about tick 22.

Fix: the cascade is only seen. A staggered control takes input as soon as the
panel's own entrance, still inert while it moves, has played. The probe taps
Play on the tower-defense title at each time and reads the gold the round
starts with:

| Build | 0.25 s | 0.267 s | 0.3 s | 0.367 s |
|---|---|---|---|---|
| 44df5d7 (before the round) | refused | taken | taken | taken |
| b2532cd | refused | refused | refused | taken |
| this fix | refused | taken | taken | taken |

## The re-recorded frames

To attribute each difference, I rendered three builds through the same
`og test`:
- 44df5d7, before the round: matches every reference within 0.010 % outliers.
- 7438978, the round alone without p2-core: reproduces every failure.
- This fix on b2532cd: within 0.010 % of 7438978's frames, the scratch builds'
  run-to-run noise.

So every changed pixel comes from the UI round, and the stagger fix and
p2-core's template changes move none. Measured against the reference, each
difference lies in a row of text. Before and after for each frame:
`rerecord/archetype-<template>-<frame>.png` (old reference left, new frame
right).

| Template | Frame | Outliers, RMSE | Why it changed |
|---|---|---|---|
| idle | title | 0.387 %, 0.0282 | "Spark Orb" at displayLg: the tokens' tracking (row 17, -0.02 em) tightens it and moves its two ornaments; the skin's `textGlow display 2.0` is now the distance-field halo (row 18), not four offset copies |
| idle | orbit | 0.196 %, 0.0102 | the HUD's amount (xl, display font, -0.01 em) and each generator's mono count and cost: the skin's `textGlow numeric 1.2` is the new halo |
| idle | card | 0.157 %, 0.0142 | the supernova card's heading (tracking) and its numeric values (the new halo) |
| rhythm | title | 0.356 %, 0.0278 | "Three-Lane Beat" at display size: tracking and the skin's `textGlow display 1.5` as the new halo, its chevrons moved with it |
| rhythm | results | 0.098 %, 0.0145 | the "Song cleared" heading's tracking and the grade "S" (display size, glow) |
| roguelike | title | 0.235 %, 0.0291 | "Lamp Dive" at displayLg: tracking only (no text glow in lampdive), its ornaments moved with it |
| roguelike | over | 0.230 %, 0.0248 | the summary's heading "The lamp went out on floor 1" (xl and up): tracking, its ornament moved |
| tower-defense | title | 0.297 %, 0.0334 | "Path Guard" at displayLg: tracking, its ornaments moved |
| tower-defense | end | 0.136 %, 0.0178 | the end card's heading and the score "520" (displayLg): tracking |

Not re-recorded: rhythm's highway, roguelike's dive and tower-defense's wave
(within tolerance); match3, runner, topdown, fx-showcase, game and ui-kit
(unchanged). With the fix and these references, `og test` passes on all of
them: idle 10, rhythm 11, roguelike 9, tower-defense 9, match3 6, runner 8,
topdown 7, fx-showcase 18, game 2 and ui-kit 29 suites.
