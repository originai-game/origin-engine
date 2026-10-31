---
name: visual-acceptance
description: Judging a game's visuals on Origin Engine against the chosen direction, each asset's own references and the frame as it plays — numbers before eyes, regressions against baselines captured the same way, individual and whole-frame judgment at the play camera and the player's viewports, fixing the stage that failed, and what a visual claim must record. Use for visual critique, art or integration review, before-and-after comparisons, or before calling a visible change done.
---

Judge the image the game actually drew against the criterion it was made
for and the references it was made from. Technical render correctness,
likeness and playable composition are different claims; say which one you
are making. No number of clean reviews makes a frame acceptable: record the
evidence and the defects still open, against the exact build.

The engine's frame tools (`og run --commands`, `describe`, `expect`,
`og compare`, the probe's `capture` and `sweep`) are og-build §1, §2 and
§4, og-game §9 and og-model §3; this skill is how to use them as a
reviewer. The look itself (the style, colour, hierarchy) is og-taste's.

## 1. Numbers and regressions first

Use the numbers the engine gives for positions, velocities, timings,
budgets and events (dumps, `get`, `describe`'s boxes, `state()` timings).
Check that physics, visuals, the camera's framing and the UI agree with each
other, rather than asking whether each looks plausible on its own.

For a rendering regression, capture the same scripted tick, viewport and
camera as the baseline and compare them: `expect:frame:<reference>:rmse:<=N`
in the script, or `og compare` for a silhouette and colour difference with
one sheet to read. A comparison detects change, not taste; open the frames
for any claim about art. Re-record a baseline only for an intended change,
with the reason written beside it, never to make an unexplained difference
go away.

## 2. Individual and whole-frame judgment

Compare each asset against its own references: silhouette and interior
landmarks, proportions, colour and material regions, detail, deformation
and motion. A mood board alone cannot specify a hero model (og-model §2 has
the detail inventory).

In a game with a look, each asset is also judged against the art
contract it was requested under (og-taste §1, §12): the art brief that
made it (its palette, pixel grid or texel size, line, face budget) is the
criterion, and the style's preview and the game's accepted assets are its
references. Read what the cook changed (`og cook --report`, og-build §8)
before reading the frame: a sprite the cook halved or a model it cut to a
fifth of its triangles was requested off the contract, and the fix is the
request, not a looser look. Every opt-out (`conform off`) is a claim to
check: say why that asset may break the style. `og style check`, when the
engine you pin has it (og-build §8), gives the contract's numbers per
asset and per frame; numbers cannot judge a style (og-taste §11), so open
the frames too.

Then look at it in the game's own camera and lighting, at play scale, and
judge the whole frame against the chosen direction: massing, palette,
atmosphere, ground cover, readable navigation, threats, interaction prompts
and how much of the screen the HUD takes. A scenic angle cannot certify a
chase camera. A local improvement that harms the whole frame fails.

Check the viewports players use (a phone's portrait and landscape are
different evidence, as is a desktop window) and every state the change
touches beyond the default: menus open, a toast showing, the hit state,
motion and transitions. UI artwork alone does not prove text layout,
touch reach or how the screen behaves. Match camera, scale and conditions
when comparing builds; state any difference you could not remove instead of
treating two unrelated pictures as a before and after.

## 3. Fix the stage that failed

When the direction is wrong, settle it before polishing anything locally.
When a model or a layout is wrong, name the criterion it fails and fix the
stage responsible (the reference, the spec, the build, the look, the
camera). Do not inflate asset counts, repeat reviews until one passes, or
soften the criterion to get a pass. Ask for an independent critique for a
concrete uncertain claim, then reproduce what it found in the running game.

Where alternatives need a choice (a colour, a fog density, a camera
height), the probe's `sweep` captures each value from the same snapshot at
the same tick, side by side: the frames are evidence and the choice belongs
to whoever owns the direction.

## 4. What a visual claim records

Record the reference and its version, the build or commit judged, the
camera, viewport, tick and seed, the exact command, the numbers it printed
and the path of every frame you opened; for an asset, the brief it was
requested with and what the cook's report said it changed. A frame nobody opened is not
evidence, and a frame from a debug camera is not the shipping camera. A
later edit makes the conclusion stale; it does not make it retroactively
true.

Frames from different renderers (a GPU, a CPU fallback such as lavapipe, a
phone's browser) differ enough that a comparison across them is a new
baseline, not a regression; compare like with like, and say where a frame
went unchecked because nothing rendered it (og-build §3). Keep refuted
attempts and measured comparisons with the project's notes. An unknown
measurement stays unknown, and a difference inside the measurement's noise
is not an improvement.
