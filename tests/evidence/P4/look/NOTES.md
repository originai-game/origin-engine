# P4 look round (r-look): phase A

Phase A lands looks and the stylisation fields as data, and the style
catalogue; no renderer pass draws a Stylise or MaterialStyle field yet
(`og_render::STYLISE_DRAWN` is empty, `RenderCapabilities.stylise` is
`[]`), so no frame changed and none was re-recorded.

## What landed

- `type look` (`crates/og-cook/src/look.rs`, `src/cook/look.rs`): grammar,
  `extends` (the game's own `assets/looks/<slug>.ogt`, else a built-in
  preset, else an asset id), flattening, checks against
  `og_api::host::RECORDS`, sRGB hex colours stored linear; `og.Look` and
  `og.Palette` blobs (`.hex`, `.gpl`).
- IDL: `Stylise`, `MaterialStyle`, `DitherPattern`, `Palette`, `Look`,
  `LookInfo`, `RenderSettings.exposure`, `RenderCapabilities.stylise`,
  `og.load.look/lookInfo/palette`, `og.render.applyLook/look`,
  `GameDesc.look`.
- og-runtime: load, apply (whole-resource replace through
  `write_resource_value`; `PostEffects.flash` and `shakeTrauma` kept; the
  skin through `og.ui.configure` with the last UiDesc), `GameDesc.look`
  before start and its antialias before the host attaches the renderer;
  extraction into og-render's `Stylise` description.
- `packages/knowledge/looks/catalogue.json` (37 styles) and og-taste §12.

## Support today (catalogue `look.support`)

full 6, partial 16, planned 6, content 9 (of 37). Phase B moves the
planned and most partial entries once the shader primitives land.

## Tests

- `cargo test -p og-cook --lib look`: 9 tests (grammar, merge, presets vs
  `templates/looks`, field checks against the records, cycles).
- `packages/engine/tests/look.test.ts` on `tests/fixtures/look`: 4 tests,
  49 expects, passed on the native host before and after the merge of
  main 376f3cd (addon rebuilt by `bun run build`).
- `packages/knowledge`: validator and 21 tests (two new: catalogue
  coverage, refused entries).

## Gate and sizes

`scripts/check.sh quick` after the merge of main 376f3cd: PASS fmt,
clippy, drift, bun-quick, test-quick (gate PASSED 5/5). The first run
failed og-idl's coverage test (the load module's function list); it now
names look, lookInfo and palette and the two resources.

`scripts/check.sh wasm-build wasm-opt size-gate` (`wasm-size-phase-a.json`),
brotli-q11, against main 376f3cd's `tests/evidence/P4/cook/wasm-size-after.json`:

| download | main 376f3cd | r-look phase A | delta |
|---|---:|---:|---:|
| 3D publish | 2,913,544 | 2,927,010 | +13,466 |
| lanes | 3,290,490 | 3,303,164 | +12,674 (78.8 % of 4 MiB) |
| 2D publish | 2,749,599 | 2,763,543 | +13,944 |

The growth is the look parser and runtime, the palette loader and the
two resources' records. The budget is unchanged.
