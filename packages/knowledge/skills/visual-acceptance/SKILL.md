---
name: visual-acceptance
description: Judging a game's visuals on Origin Engine against the chosen direction, each asset's own references and the frame as it plays — numbers before eyes, regressions against baselines captured the same way, individual and whole-frame judgment at the play camera and the player's viewports, fixing the stage that failed, what a visual claim must record, the art bar a frame that passes every check is still held to (hierarchy, real art, numerals, selection, depth, spacing, motion stills, each relative to the game's own style and decided by a number og frame measure, og ui check or og style check reproduces), and how a review rules (approve or hold, each hold a frame, a region, a criterion and a measured value against what passes). Use for visual critique, art or integration review, an art director's verdict, before-and-after comparisons, or before calling a visible change done.
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

## 5. The art bar

The checks of §1 to §4 catch defects; a frame can pass every one of them
and still read as a prototype. A frame is also held to this bar. Every
item is relative to the game's own look: its style's checks, its art
bible, its look's contract (`og style check --json`'s `contract`) and
what the style means (the catalogue entry's `intended`, og-taste §12).
A trait the style means is never a failure of the bar: hard one-pixel
steps in a pixel style, four colours in a limited palette, broken edges
in a painterly one, a near-black foreground in a silhouette style. Each
item names the number that decides it and the command that measures it,
so a reviewer's hold can be measured again by anyone:

- **`hierarchy`: one dominant element per screen and per control.** The
  eye reads one thing first: the screen's primary action, the HUD's
  vital, a control's label over its decoration. It leads its peers by
  size or by contrast against its ground, not both left equal: a hero
  slot about 1.15× its row's other slots, the primary's contrast against
  its ground (`og frame measure --metric contrast`) above every
  secondary's on the same screen. Two elements tied for first is a hold.
- **`art`: real art, never a placeholder.** No art slot still drawing its
  style's placeholder (`og ui slots --json` marks each `art:
  "placeholder"`: the count passes at 0), no debug letters or initials standing in for an
  icon (N, H, R in empty slots), no flat fill where the screen's concept
  has a picture. Measured as an occurrence count.
- **`numerals`: gameplay numbers read at a glance.** Damage numbers,
  timers, scores and counts drawn over the scene carry a 1 to 2 px
  outline or shadow in the style's own ink, and meet the existing
  contrast bar on the ground they sit on: og ui check's for UI text,
  else 3:1 against the scene behind (`og frame measure --region <the
  digits> --against <the scene beside them>`).
- **`selection`: selection and focus visible against the style's own
  outline.** A selected or focused part differs from its idle state by a
  mark the style's ink cannot hide: never a bare 1 to 2 px white or
  ink-coloured outline (a navy ring vanishes on a navy-inked style). Its
  mark reaches 3:1 against the ground beside it (WCAG 2.2's focus
  appearance; `og frame measure --metric contrast` over the mark
  `--against` the ground), and its colour is not `ArtLine.color`.
- **`depth`: depth and material as the style's layers define them.**
  Panels separate from the scene the way the style's layers do (a plate,
  a sheet, glass, a cut edge), and a flat style stays flat. Measured as
  the step in mean grey between the panel and the scene beside it
  (`--metric mean`), or the high-pass energy of a surface against a
  reference region (`--metric highpass --against`): a material the style
  textures shows energy, one it fills flat shows none.
- **`spacing`: one rhythm.** Gaps between siblings of a row or column
  come from one spacing scale: within 1 px of each other, or a step of
  the scale apart (the boxes in `og ui check --json`'s findings and
  `dump_ui`'s elements give them in CSS px).
- **`motion`: motion stills hold.** A screen with a transition has a
  mid-transition still in its capture script (`…;1.2:screenshot`); the
  still shows no torn, doubled or off-palette frame. A fade or gradient
  steps by no more than about 0.03 grey per 10 px band over its run
  (`og frame measure --metric ramp --rows 10`), and a cooling or
  disabled part is dimmed to a stated share (a cooldown sweep at least
  55 % black, `--metric mean` against the slot when ready).

`og style check --sizes desktop,phone --json` captures each declared
screen at both sizes, each frame labelled `<screen>@<viewport>` with its
size and renderer tier, and names any scene with no capture script:
judge every item on every one of them, and each tier against its own
(§4).

## 6. Verdicts

A review, by the author before asking for one or by an independent
reviewer, ends in one of two verdicts:

- **approve**: nothing fails a measured standard, the style's checks,
  the art bible or the art bar (§5).
- **hold**: one or more holds. Each hold names a **frame** (its
  `<screen>@<viewport>` label), a **region** (a box in fractions of that
  frame, `x,y,w,h`), a **criterion** (`style-check-N`, the style's check
  as written; `bible:<clause>`, a clause of the art bible; `contract:<term>`,
  a term of the look's contract with its value; `measure:<name>`, a
  number an engine command printed, against the bar the command states;
  `bar:<item>`, an item of §5 by its id), the **measured value now** and
  the **condition that passes it**, and how to measure it again (the
  `og frame measure` line with its region, or the `og ui check` or `og
  style check` field). "Make it pop" is not a hold; "the cooldown sweep
  dims the slot to about 20 % black; at least 55 %, with a 1 to 2 px
  bright leading edge" is.

How the holds are ruled:

- **A measured failure is not a preference.** A preference (a colour you
  would choose, a polish no number decides) is a note, never a hold, and
  never blocks.
- **Contrast is judged at the bar the engine's check states for that
  element, never stricter**: og ui check's 3:1 for live UI text (7:1 in
  high contrast; under 4.5:1 is its warning, a note unless the art bible
  asks for more), og polish's bars for what it measures. Text over the
  game scene that no check measures is the reviewer's eye: the note says
  so, and quotes the `og frame measure` ratio it read.
- **The style is the criterion.** Nothing the catalogue's `intended`
  names, nothing the look's contract sets, and no trait of another style
  is held: a game is never held to a different style or to the
  reviewer's taste.
- **A resubmission is judged on the held notes and on regressions
  only.** Each earlier hold is fixed or still held, with its new
  measured value; a new hold is allowed only for a region approved
  before that regressed. No new taste notes in the resubmission.
- **Reviews are not repeated until one passes** (§3). A hold still
  standing after its resubmission goes to whoever owns the direction,
  with its frames and numbers, rather than round after round.
- **A verdict is a visual claim** (§4): record the build, the frames,
  the numbers and the commands that measured them.
