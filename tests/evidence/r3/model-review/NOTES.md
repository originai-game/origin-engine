# r3/model-review

Branch `r3/model-review` from origin/main at b86bcb91. One commit per item:

| Commit | Item |
|---|---|
| 6e80909 | og-runtime: `view` and `turntable` review cameras (command channel) |
| e5ddb4d | `og compare`: silhouette IoU, ΔE00 per region, the review sheet |
| 7fa5f11 | knowledge: the `og-model` skill; docs/design.md §2.11 |

## What landed

- `view:<entity>:front|back|side|top|orbit[:<deg>]`, `view:off`,
  `turntable:<entity>:<n>[:<prefix>]` (crates/og-runtime/src/review.rs),
  registered beside `debug` (e6.rs), not built into publish. The view frames the
  world box of every instance drawn for the entity and its og.Parent descendants,
  orthographic for the four presets (front from +Z, back from -Z, side from +X,
  top from +Y with the front at the bottom), orbit a 30° perspective 20° up;
  10 % margin each side. It is RenderState: og.Camera is never written, so no
  snapshot, replay or hash sees it. Bounds from the last frame's instances
  before LOD selection, and again from this frame's once every instance is in.
  A turntable queues n (≤ 72) view+screenshot pairs for the next tick's start
  through the probe inbox, all on one tick, then restores the view before.
- `og compare <ref> <render> [--mask] [--align] [--key] [--sheet] [--json]`
  (crates/og-cli/src/compare.rs): silhouette = alpha ≥ 128 if the image has
  transparency, else ΔE00 > key (8) from the median border colour; IoU; CIEDE2000
  (sRGB D65) over the pixels both cover, mean and p95, per flat mask colour (each
  with its own IoU) or over the overlap; one sheet PNG (reference | render as
  compared | silhouettes overlaid: grey both, red reference only, blue render
  only) labelled in the contact sheet's pixel font (glyphs D E F I O R U added).
  Render resampled when the aspect agrees within 1 %, refused otherwise (names
  `og run --size WxH`); `--align` matches silhouette boxes. Deterministic.
- `packages/knowledge/skills/og-model/SKILL.md`: intake (reference admission,
  3×3 detail inventory mapped to parts and generators, region mask, hidden
  sides), spec, loop with the five decisions and 3/6 correction limits, the
  review frames and what the numbers cannot see, the eight passes with gates,
  generator recipes by form plus a lantern using every generator (run and
  reviewed, below), budgets, reporting. Self-contained: og-game §18 is not
  edited; og-game §1's skill table gains one row.

## How each test fails on the old code (checked)

- `crates/og-runtime/tests/review_views.rs`: projects a 2×1×0.5 box through the
  camera each capture was drawn from (a recorder backend). Checks orthographic
  collapse (4 distinct corners), the binding extent at 1/1.2, top's height =
  front's × depth/height with the same width, side's width the depth, each
  view's orientation, a child framed with its parent (height 2.4, centre 1.5),
  og.Camera's frame back after `view:off`, a 4-frame turntable at azimuths
  0/90/180/270 and the prior view restored. Checked by unregistering the
  commands: fails with "unknown action 'view'". Unit test in review.rs: the
  orbit fits the bounding sphere at three aspects and four azimuths.
- `compare.rs` unit tests + `tests/cli.rs::compare_prints_...`: ΔE00 against
  Sharma–Wu–Dalal published pairs (8 pairs, 5e-5), Lab of sRGB primaries; a frame
  against itself IoU 1, ΔE00 0; a 10 px shifted recoloured square IoU 0.6 and the
  pair's ΔE00, per-region pixel counts and IoUs (800/1200, 400/800) by hand;
  `--align` undoing shift + 2× scale; a cut-out's alpha; the sheet's panels.
  On the old code `og compare` is an unknown command (exit 2): the CLI test fails.
- `packages/knowledge/tests/validate.test.ts`: the pack list includes
  `skill:og-model`, and a new test holds og-model to the review's commands,
  flags, generators and eight passes and shows a misspelt generator refused.
  Checked by removing the skill: both fail, validate prints
  "skills/og-model/SKILL.md: missing".

Loop run: `cargo test -p og-runtime` (all pass), `cargo test -p og-cli --bin og`
and `--test cli compare`, clippy -D warnings on both crates (also og-runtime
`--features publish`), fmt, `og api check` (api.ogt unchanged),
`bun scripts/validate.ts` and `bun test tests/` (44 pass).

## Frames read (lavapipe, `og run --commands`, OG_NATIVE_ADDON = dev addon)

- `views-lavapipe.png`: a lamp (lathed column, extruded arm to +X, gear with
  three holes) — og.Camera's own frame (small, off-centre), then front (arm
  right, gear facing), side (arm end-on, gear edge-on, the column centred), top
  (the foot's disc, arm to the right, the front at the bottom), back (mirrored:
  arm left). Each framed with the margin.
- `turntable-lavapipe.png`: 8 orbit frames 45° apart, the lamp centred and the
  same size in each, the arm sweeping round.
- `compare-front-back-lavapipe.png`: front vs back, IoU 0.2428, ΔE00 16.78; the
  overlay shows the arm grey and each column/gear red or blue. Front vs itself:
  IoU 1, ΔE00 0.
- `compare-front-orbit-align-lavapipe.png`: front vs turntable frame 0 with
  `--align`, IoU 0.7398, ΔE00 4.77; the foot's ellipse blue, the flat foot red.
- `lantern-pass1/2-lavapipe.png`: the skill's lantern, front/side/top/orbit.
  Pass 1: the handle floats above the cap and the crest stands off the cap in
  side view. Pass 2: handle ends on the cap, crest on the slope, all views agree.

## Metal references owed

None: no og-tests gfx scene added, no reference frame moved (with no view set,
extraction is unchanged: `review::bounds` returns at once).

## Shared files touched

crates/og-runtime/src/{extract.rs, geometry.rs (bounds cache shared as
`mesh_bounds`), e6.rs, lib.rs}; crates/og-cli/src/{main.rs, contact_sheet.rs};
packages/knowledge/{scripts/validate.ts (SKILLS), tests/validate.test.ts,
README.md, package.json ("eleven"), skills/og-game/SKILL.md (§1 table row
only)}; docs/design.md (new §2.11). og-cook and og-game §18 untouched.

## Left out / for the coordinator to decide

- `view`/`turntable` are registered commands (like `debug`), not in api.ogt's
  `commands.CoreAction`; lifting them into the IDL (and the wiki's commands
  page) is a decision.
- A skinned model's posed parts and sprites are not framed.
- No img2threejs-style state file or automated gate runner; the spec and
  decisions live in `review/<model>/spec.md` by the skill's convention.
- og-build's loop does not mention og compare yet; og-model is the one place.
- ΔE00 includes the render's lighting; the skill gates material ΔE only from
  pass 4 and records it ungated in pass 6.
- Once geom-2's `type model` lands, the skill's recipes could add the `.ogt`
  route; kept out to avoid conflicting with §18.
