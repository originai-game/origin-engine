# E7: each change's test broken once

Every item closed in the E7 round carries a test that was run against the
change undone (or a defect put back) to see it fail, then against the
change to see it pass. Owed-table rows are docs/phase-1.md §Owed.

| Row | Change | Broken how | What failed | Commit |
|---|---|---|---|---|
| 1 | `og.ui.shake` | og-ui's `end()` without `apply_shakes` | og-ui skin `game_code_shakes_an_element_once_and_it_comes_to_rest_by_itself` | da47e5e |
| 2 | tiled nine-slice edges | og-ui's `tiles()` always one stretched piece | og-ui skin `a_slice_tiles_its_edges_and_centre_when_the_skin_says_repeat_or_round` | 4a19260 |
| 3 | phone references for the six packs | skins.ts's phone surface without `viewport:phone` (laid out as a desktop) | `og test templates/ui-kit`: the six `skin-*-phone` suites (6 of 25) | d7c95b3 |
| 4 | camera zones scroll | feel.rs's `camera_zone` never starts a transition | topdown `door`: "the view scrolls between the rooms for several samples (0)" | 0c630ce |
| 5 | resource commands, usage in help | og-probe `set` without the resource branch; engine.rs ignoring `GameDesc.commandHelp` | og-probe `set_and_get_reach_resources_entities_components_and_fields`; runtime.test.ts "GameDesc.commandHelp gives…" | cea6fab |
| 6 | `getTile` on a terrain layer | og-tilemap's `get` answering the terrain bit again | og-tilemap autotile test | 0006005 |
| 7 | ragdoll animation targets | og.ragdollTargets computing the pose but never handing it over | anim.test.ts "a ragdoll at weight 0 follows…" (bones still: 0) | c6d5793 |
| 8 | runner on og.PlatformerController | the body's `jump` forced to 0 | runner `jump`, `replay`, `sounds`, `tune` | 142b9da |
| 8 | topdown on og.CharacterController | the hero's controller with `gravityScale` 1 (not the top-down mode) | topdown `walk` | 497deee |
| 9 | kinsoku line breaking | og-ui's `tokens()` without the kinsoku glue | og-ui linebreak `kinsoku_keeps_…` | ba1042a |
| 10 | polish: text only, at the surface's pixels | polish.ts scaling by the device size again; disabled ancestors not consulted | polish.test.ts phone fixture (texts 0; texts 3) | 891b4b6 |
| 11 | fx-showcase builds E5 stations lazily | `buildE5` building every station at start | fx-showcase `frames`: "no E5 station was built (727 entities)" | 93d86e1 |
| 12 | templates typechecked in the gate | fx-showcase's `hud(font: number)` | `bun run typecheck` (tsconfig.templates.json) TS2322 | c5295be |
| 13 | native and Chrome world hashes | `og run` starting the game before the renderer is attached | tests/web/archetype-hashes.ts: runner and match3 differ from Chrome | 85c2bd0 |
| 37 | the Player protocol the platform landed | shell/platform.ts sending revision 0 for an unconditional `store.set` | platform-player.test.ts keyed store (the second `og.settings` save conflicts) | 96f4ebd |
| — | skills as one set | the knowledge validator's `coherence()` not called | validate.test.ts "each module is taught by the one skill…" and "a section reference names a numbered section…" | 34829c6 |
| — | each IDL module names its skill | og-idl's missing-skill check off | og-idl `broken_idls_are_refused_with_the_right_reason` (missing-skill.ogt accepted) | 34829c6 |
| — | polish's fixes name existing sections | a fix naming og-feel §31 | polish.test.ts topdown (unresolved og-feel §31) | 34829c6 |
| — | polish's unanswered hit | the hit-stop count taken as every damage moment | polish.test.ts "a damage moment without hit stop is a finding…" | 62f52f3 |
| — | polish reads screens as screens | each of: the modal/dialogue/list rule off, the scrim exclusion off, touch controls counted | polish.test.ts "og polish reads a dialog as a screen…" (the loading screen counted as play; 5 texts read under the dialog, not 4; play HUD over 3 %) | da05a4d |

Every break was undone by restoring the file from a copy taken before it
(never `git checkout` or `restore`), and the same test then passed.
