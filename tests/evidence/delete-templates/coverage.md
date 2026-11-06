# Phase 2: the gameplay templates go (plan and coverage, WIP)

Status: **not started, plan only** (2026-11-06, delete-examples thread
T-01a1b07f). Phase 1, examples/, landed at 876766dc0. This file lists what
reads each phase-2 item and where each check moves, so a fresh thread can do
the work. Nothing below is deleted yet.

The owner's brief (relayed by T-38e88112, revised 2026-11-06):

- `og create` writes one project skeleton: project.ogt, `src/` with an empty
  game, tsconfig, package.json, `tests/` and `assets/`. There is one skeleton
  for 2D and 3D alike.
- The creation choices go into project.ogt's `create` block as constraints
  and hints, never into code or assets.
- Delete templates/archetypes, templates/fx-showcase and the ui-kit gallery
  showcase.
- templates/game and game-2d stop being games. Their content moves into the
  fixtures the tests need: it is moved, not rebuilt.
- Keep templates/styles (the UI packs), the kit code, templates/looks,
  templates/locales and packages/design.

## A. Readers outside the templates (rg, 2026-11-06)

| Item | Who reads it | Where it moves |
|---|---|---|
| templates/game `assets/models` (dropship.glb, walker.glb) | og-cook `impostor.rs`, `raycast_bvh.rs`, `stream.rs`, `compressed.rs`, `template.rs`; og-runtime `crowd.rs`, `garment.rs`, `model_clips.rs`, `render_raycast.rs`, `shadow_flags.rs`; packages/engine `support/anim-hash.ts`, `og-render.test.ts`; og-anim `tools/make_rig.py` | `tests/fixtures/models-3d/` (git mv the glbs and their sources; one shared asset fixture for the Rust tests, no playtest) |
| templates/game `assets/ai`, `nav` | og-ai `tests/ai.rs` | `tests/fixtures/ai-nav/` |
| templates/game `assets/textures`, `audio` | og-cook `textures_p4.rs`, `stream.rs` | `tests/fixtures/models-3d/` (same fixture) |
| templates/game `assets/fonts` | check.sh `rule_gpu` (og-ui, og-tests-cooked) | the ui-kit fixture or `templates/styles` (the fonts the packs embed) |
| templates/game tests (guard, inspector, inspector-gizmo, intro, smoke) and its 7 references re-recorded at b54e9e5a2 | og test in the template; meshlet determinism (T-01a1b08f); og-session's template baseline; invariants' conversions | `tests/fixtures/dropship-scene/`: the dropship scene moved verbatim with guard + inspector + one frame; `tests/fixtures/meshlet-determinism/` if the repro needs its own |
| templates/game-2d | `bundle-hash.test.ts`; og-tests `rig2d/keeper.ogt`; `tests/replay/golden/goldens.json`; tools/art-samples (2D profile) | `tests/fixtures/sprite-2d/` (keeper rig, the 2D bundle hash, the replay golden) |
| templates/game + game-2d as create's output | packages/create `create.ts`, `pack-template.ts`, `tests/create.test.ts`; packages/knowledge `scaffold.ts`, `mount.ts`, `scaffold/*.json`, `tests/mount.test.ts`; og-cli `check.rs`, `render.rs`; tsrs-corpus tsconfig | one skeleton in `packages/create/template/` (template-2d goes), written by create.ts |
| templates/archetypes/arena (online lockstep duel) | `rooms.test.ts`, `web-rooms.test.ts` | `tests/fixtures/lockstep-room/`: two peers, one input each, og.assert that both worlds' hashes agree every tick |
| templates/archetypes/roguelike (DotGothic16) | og-cook `font.rs` pixel-font test (retargeted there in phase 1) | `tests/fixtures/pixel-font/`: the font file and its licence only |
| templates/archetypes/topdown | og-runtime `sprite_arrays.rs`, `stream_release.rs` | `tests/fixtures/sprite-2d/` |
| templates/archetypes/* styles (sparkorb, lampdive, lanebeat, pathguard, ember) | check.sh `rule_gpu`; og-tests-cooked style diagnostics; og-taste's contrast note | **dropped** with the games (their styles were game styles); the packs under templates/styles keep the contrast suite |
| templates/archetypes/third-person | web-reference's `ember-vale` group, rule_web | `tests/fixtures/vehicle-terrain` already covers a 3D scene in the browser; the third-person controller gets `tests/fixtures/character-3d/` (capsule walk, step, slope, og.assert grounded) |
| archetypes in create (genre/archetype pick, overlays) | `create.ts`, `apply-choices.ts`, `create.test.ts`, `release-bump.ts`, `pack.sh` dist layout | dropped from the output; the genre becomes a `create` constraint line |
| templates/fx-showcase textures, models, baked scenes, motion | og-cook `retarget.rs`, `physical.rs`, `stream.rs`, `anim_compress.rs`; og-runtime `baked_scenes.rs`, `material_arrays.rs`, `model_lod.rs`, `motion_performance.rs`, `sprite_arrays.rs`, `stream_release.rs`; og-anim `make_humanoids.py` | `tests/fixtures/materials/` (PBR textures and arrays, LOD model), `tests/fixtures/motion/` (retarget, compression, motion performance), `tests/fixtures/terrain-stream/` (streaming groups, terrain), `tests/fixtures/baked-light/` |
| templates/fx-showcase playtest (post.png, stylised.png) and web `fx` group | og test; `web/reference.ts`; `frames-vs-metal.ts`; Makefile | `tests/fixtures/effects/` (particles, post) and `tests/fixtures/weather-water/`, one frame each; the web `fx` group points at `effects` |
| fx-showcase stations for 3D fluids, cloth and ragdoll | station playtests (already cut to 2 frames by test-restructure 316091a12) | `tests/fixtures/fluids-3d/`, `tests/fixtures/cloth-ragdoll/`, invariant-only (no frame) |
| templates/ui-kit gallery (race scene, starship, genre showcases, 570 assets, 64 kit frames) | og-ui `features.rs`, `glyphs.rs`, `rtl.rs`, `runs.rs`, `text-fonts`; og-tests-cooked `text_gfx.rs`, `text_outline.rs`, `materials_gfx.rs`; og-cook `strings.rs`; `i18n.test.ts`; web `dogfood.ts`, `published.ts`, `reference-gfx.ts`, `text-fonts.ts`; og-cli `ui/sheet.rs`; pack.sh; check.sh ui-styles/ui-kit/kit-web; wasm-determinism | kit behaviour: one fixture per component group (`ui-controls`, `ui-lists`, `ui-dialogs`, `ui-text`), each one frame plus og.assert focus and layout invariants, no showcase art; style system: the packs' own suites stay (`templates/styles/tests`); strings and fonts move to `ui-text` |

## B. Dropped (proposed)

- The archetypes' and the game template's game logic (genres, scoring,
  runner coins, match-3 rules). These are game code, not engine features.
- Per-archetype styles, the race scene and the starship art. They were game
  content; the UI packs stay.
- Most of the 64 kit frames: one frame per component group remains.
- The genre, view and starter overlays in create. The choices remain only as
  constraints.

## C. Changelog breaks to add (CHANGELOG.md, when it lands)

- `@origingame/create`: `og create` writes one project skeleton plus a
  `create` block of constraints; it no longer copies the game, game-2d or
  archetype templates. `--archetype` and the 2D overlay are removed.
  `applyCreation` writes no code or assets.
- Dist layout: `create/` holds only the skeleton; `archetypes/`,
  `templates/game`, `templates/game-2d`, `templates/fx-showcase` and the
  ui-kit gallery are no longer in a version's archive.
- Removed templates: archetypes (nine), fx-showcase, the ui-kit gallery
  project, game and game-2d as games.

## D. Platform ask (draft; T-38e88112 relays it to origin-agent)

> The engine's next minor version changes what `og create` writes. A new
> project is one skeleton: project.ogt, `src/main.ts` with an empty game,
> tsconfig.json, package.json, `tests/` and `assets/`. There is no 2D or 3D
> variant, no archetype and no sample content. The creation choices (genre,
> view, art style and mood, UI style, platform and input, players, languages,
> music) are still chosen by the agent and passed to create as today. They
> are written into project.ogt's `create` block as constraints and hints that
> the agent and the skills read. They produce no code or assets.
>
> Please make three changes on your side:
> 1. Stop relying on template content: the dropship scene, game-2d's keeper
>    and the archetypes' screens.
> 2. Have the agent build the game from the skeleton with the engine API and
>    the skills, reading the `create` block.
> 3. Update any probe or Stage expectation that assumed a template's first
>    frame.
>
> The choices schema keeps its version number and gains the bilingual labels
> from the paused platform-scaffold-contract branch (6a62b9cd0). That
> branch's 2D overlay is dropped. The archive no longer has `archetypes/`.
