# Kit reference-frame reduction

Base: `5387f79db` on origin/main, after the genre move landed at `5bbbba76e`.
Branch: `kit-frames-cut`. References are **deleted deliberately**, not missing
assets to regenerate. The preceding move remains a separate change.

## Inventory

Counts are PNG files. A mixed showcase sheet counts once under its primary
genre family. The retained filenames and owners are explicit in
`templates/ui-kit/tests/reference-frames.ts`. All 64 retained files match the
base byte-for-byte; no remaining image or comparison tolerance changed.

| Owner | Before | After |
|---|---:|---:|
| card | 91 | 5 |
| flight | 0 | 0 |
| gacha | 39 | 3 |
| idle | 7 | 3 |
| liveops | 39 | 3 |
| materials | 2 | 2 |
| photo | 18 | 3 |
| puzzle | 18 | 3 |
| racing | 39 | 4 |
| rhythm | 18 | 3 |
| rpg | 126 | 5 |
| shooter | 22 | 3 |
| sports | 4 | 3 |
| story | 38 | 4 |
| strategy | 110 | 5 |
| versus | 73 | 5 |
| UI template, generic | 257 | 5 |
| style/fantasy | 2 | 2 |
| style/scifi | 2 | 2 |
| style/storybook | 1 | 1 |
| Each other style pack | 0 | 0 |
| **Total** | **906** | **64** |

The other style packs are broadcast, cartoon, craft, cyber, dread, foundry,
gilded, kawaii, pixel, tactical and wasteland. Their style parsing tests stay.
Materials already had only two surface references and flight had none; no new
image was manufactured to fill a quota. The five style sheets already fit the
small per-pack PNG budget and remain unchanged.

Selection favors desktop/phone main screens and visual states: card targeting,
legendary reveal, hold progress, reward claims, camera framing, puzzle stars,
checkpoint composition, drag rejection, attachment layout, gallery/route views,
targeting overlays and separate player focus rings. Generic coverage retains
desktop/phone widgets, composition underline, a submenu and a binding conflict.

## Verification

- `scripts/check.sh ui` before and after: **109 pass, 2 fail, 874 assertions**.
  The unchanged failures are RPG unitFrame/partyFrames text and racing
  speedometer text. Neither is fixed or waived here.
- Run separately after that expected stop: `scripts/check.sh ui-styles ui-kit`
  passes all **5 style suites and 44 template suites**, before and after.
- The actual after-run log compares all **59 retained kit/template PNGs**.
  The five style suites compare the retained style sheets.
- All 49 bundled playtests have identical setup, viewport size and complete
  non-screenshot command strings, including timestamps/order. Every retained
  capture stays at its original timestamp. Screenshot commands fall from
  **1,104 to 90** (PNG counts differ because sheets contain multiple captures,
  and some screenshots support non-reference checks).
- The 23 changed check callbacks retain the same non-frame bodies after
  removing frame assertions for comparison. The only additional normalization
  drops an unused showcase tuple binding. All behavioral assertions stay.
- Engine typecheck passes. Comparison functions/tolerances are unchanged;
  empty selected lists do not enter the reference checker. A missing retained
  reference still fails.
- Browser selection no longer accepts the removed locale image matrix.
  Browser rendering was not rerun; the remaining browser cases consume the
  same selected native playtest commands. No wasm/full gate was run.

## Timing

macrunner, 12 CPU cores, Rust 1.99.1, same base/native engine and default lane
parallelism. These are one observed warm run per side, not an isolated
performance benchmark. Other streams remained active: load averages at the
start were 18.95 before and 8.50 after, rising to 16.96 during the after run.

| Command/step | Before | After | Outcome |
|---|---:|---:|---|
| `scripts/check.sh ui` (warm) | 46 s | 35 s | both stop at the same two kit failures |
| `ui-styles` | 19 s | 25 s | 5 suites pass both sides |
| `ui-kit` | 581 s | 445 s | 44 suites pass both sides |
| `scripts/check.sh ui-styles ui-kit` | 600 s | 471 s | both steps pass |

There is no successful end-to-end kits-lane duration to claim: its unchanged
behavior failures stop the lane before the frame steps. Running those steps
separately avoids reporting an early failure as a speedup. The first pre-cut
`ui` run took 151 s while compiling changed base crates; the table uses its
46 s warm rerun instead. The retained interaction/state timelines still run,
so deleting most PNGs does not remove most simulation work.

Recheck the inventory and immutable bytes with:

```sh
bun templates/ui-kit/scripts/audit-frames.ts 5387f79db
```
