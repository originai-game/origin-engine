# Genre UI relocation, 2026-11-06

Base: `463343f616ba663fce0ccd78462c5c854c9a9d65`. Branch: `ui-genre-move`.
This is an ownership change, not a layout or behavior fix.

Nine new kits group callers by the feature they install: card (cards and
roguelike choices/maps), idle, liveops, story, versus (including chat), rhythm,
sports, flight and photo. Existing rpg and strategy gain worldMap and ledger;
existing kits also receive their screen documents. There are 29 moved members,
101 moved IDL types and 43 moved screen documents. Sixteen generic screens stay.
heroButton, starBar, settingsTree, overlayBar, abilitySlot/abilityBar/statusGrid
stay: the latter compose shared cooldowns, charges and effects rather than a
genre-specific game model. Rust UI primitives are unchanged.

## Verification

- Type-erased comparison: all 101 runtime declarations from the affected source
  files match the base exactly. The 124 split Bun test declarations match after
  normalizing kit namespaces and the shared test counter's module access.
- All 908 relocated reference PNGs compare byte-for-byte with their base paths:
  zero pixel difference in the reference files. No reference was re-recorded.
- Baseline and relocated Bun behavior suites both execute 126 tests:
  **123 pass, 3 fail, 991 assertions**. The failures are the same: generic
  light-style HUD text, rpg unitFrame/partyFrames text, racing speedometer text.
- `scripts/check.sh ui`: fails at kit-tests, with 109 passes and the two known
  RPG/racing failures (874 assertions). The generic light-style failure stays
  in the engine suite. Nothing was waived or fixed to make this lane green.
- Run separately after that stop: `scripts/check.sh ui-styles ui-kit` passes,
  with 5 style suites and 44 template suites. These compare against the unchanged
  references at their existing tolerances. Some captured frames have small
  nonzero deltas; this is not a claim that every fresh capture equals its
  historical reference at zero tolerance.
- Cargo check for og-api/og-ui, og-api tests, og-ui library tests (35), CLI UI
  tests (6), API drift, and the engine's three TypeScript configurations pass.
- Relocated Rust document catalogue (1) and style-pack checks (5) pass.
  All four historical contrast diagnostics remain ignored, as on the base.
- Browser kit suites were relocated but not executed; no wasm build or full
  engine gate was run in this stream.

The rendered rewards phone preview was inspected. Its lower card is still
obscured by the action region and its upgrade tag overlaps card content.
The reported 150% phone layout defect is intentionally not fixed here.

## Test ownership

`scripts/check.sh kits` (`ui` alias) owns kit behavior, document catalogue,
style packs and template playtests. `kit-web` is an explicit browser step.
Engine quick/functional/full lanes do not invoke these package suites. Optional
Rust targets use `kit-tests`; shared helpers do not register extra suites.
The concurrent test-restructure stream must preserve these package-owned tests
and references when consolidating engine tests.

## Landing correction

The landing gate caught style tests and reference PNGs inside the shipped
pack roots. They now live outside those roots at `templates/styles/tests/<pack>`
and `templates/styles/references/<pack>`; no cook exclusion or runtime change
was needed. Both previously failing cook tests pass:
`every_thread_count_cooks_the_same_bytes` and
`every_asset_type_reports_its_figures_and_its_budgets_flag_it`.
All five style suites pass again, and their five reference PNGs remain
byte-identical. This correction does not perform the later frame reduction.
