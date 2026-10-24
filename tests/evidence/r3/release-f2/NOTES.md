# release-f2: the owed feature rows 58, 59 and 63

Branch `r3/release-f2`, from `34db4fde` (origin/main). Row F2 of
`docs/release.md` §4.F. Worked in an orb (x64 Linux, lavapipe as the
headless adapter, so og-ui's native frames ran on `vulkan-cpu` and were
judged against `metal`). No browser was run.

## Per commit

| commit | row | what landed |
|---|---|---|
| `1ef30b48` | 63 | og-cook's `.vox` reads a node's `_loop` (`nTRN` and `nSHP`, `1`): its keys repeat from the first to the last on every frame outside them, a frame before the first counting back from the last, as ogt_vox's `compute_looped_frame_index`. It also reads the playback range (`META`'s `_anim_range "<first> <last>"`, MagicaVoxel 0.99.7.2): the frames walked go to its last, keys past it are not played, and it is the default clip `frames` (a `voxClip` may still name frames before it). A range ending before its start is ignored, as ogt_vox ignores it. A static file with the editor's default `0 30` stays static, byte for byte. Docs: og-cook README (the vox row and §What phase 2 owes), the vox module doc, the IDL doc of `og.load` (regenerated: `index.d.ts`, `api.schema.json`, wiki `load.md`; `og api check` clean) and og-game's SKILL. |
| `284b32c2` | 59 | og-ui: under mac keys Control+K, U and W kill into a kill buffer. Kills in a row add up (K after the buffer, U and W before it), and any other edit, move, press or typing ends the row. Control+Y yanks the buffer over the selection (a field yanks it without line breaks; one undo step). Control+T transposes the clusters around the caret (the two before it at the paragraph's end). The buffer is one for every text control. ← and → (no word key, no line key, nothing selected or Shift held) step one cluster on screen (`edit::visual_step` over `caret_slot`), so they cross a bidi run's edge to the stop next to it. A logical place at an edge has two places on screen, and the caret is drawn at the one it moved to (`State::caret_upstream`, `EditView::upstream`, `caret_x_at`). When two stops share one place, the one nearer in the text wins. At a line's edge in an area, the caret goes on to the next or previous line in the direction the line runs. New `UiKey::KeyT`. `save_state` is now version 9 (it adds the kill buffer, the kill chain and the caret's place). Docs: og-ui README (§Editing, §Input's table, the tests table, §What phase 2 owes), the edit module doc and og-ui's SKILL. |
| `7d786817` | 58 | The four cooked-font text scenes moved from `tests/text_gfx.rs` to `crates/og-ui/tests/support/text_scenes.rs` (`TEXT_SCENES`, `FONTS`, `fonts_from`, `text_scene`). og-tests-web includes that file and lists the scenes as `og-ui/text-bidi`, `og-ui/text-kerning`, `og-ui/text-edit` and `og-ui/text-mixed-runs`. The fonts' import settings are now one committed project, `crates/og-ui/tests/text-fonts` (settings only; the font files stay in templates/ui-kit). The native test cooks it and decodes the fonts from the blobs' bytes (`Bundle::from_manifest` + `Bundle::font`), the same path the browser takes. `reference-gfx.ts` cooks the same project with `og cook` (`tests/web/text-fonts.ts`), serves `cooked/` at `/game/text-fonts/`, and on every page calls `og_gfx_text_manifest` and `og_gfx_text_file` for each blob. og-tests-web gains og-cook (`runtime`), og-i18n and og-input. Docs: docs/ci.md §og-tests' gfx scenes in the browser, og-ui README. |
| this commit | — | the owed table (rows 58, 59, 63) and these notes |

## How each test fails on the old code

- Row 63, `og-cook` lib tests `looped_nodes_repeat_their_keys` and
  `the_playback_range_is_the_default_clip`. With `_loop` read as false
  and `_anim_range` ignored (the old reading, simulated by patching the
  two reads out), both fail. The first: the shape runs
  `[(0, 1..2), (1, 2..6)]` and the transform track
  `[5, 6, 6, 6, 6, 6]`, where the looped run is `[(1,0..1), (0,1..2),
  (1,2..3), …]` and `[5, 6, 5, 6, 5, 6]`. The second: `frame_count` is 3,
  where it should be 5 (the range's end plus one). All 17 vox tests and
  all 70 og-cook lib tests pass with the change.
- Row 59, `og-ui --test fields`, run on 34db4fde's `src/` with the new
  tests:
  - `mac_keys_keep_an_emacs_kill_buffer` fails at its first yank:
    `"|hello"` where it should be `" brave new world|hello"`; Ctrl+Y did
    nothing.
  - `arrows_move_visually_across_a_bidi_edge` fails: Shift+→ in
    `abc אבג` gives `["a","ab","abc","abc ","abc","abc "]` where it should
    be `[…,"abc ","abc אב","abc א"]`. The old → stepped back and forth
    across the edge.
  - With only the caret's place at the edge removed (`caret_slot`
    ignoring `upstream`), the same test fails differently: it jumps
    `"abc אבג"`, skipping the edge's second place.
  - `mac_keys_honour_the_emacs_keys` now also checks that Ctrl+T and Y do
    nothing off a Mac.
  - `arrows_move_as_they_point_in_right_to_left_runs` (the existing
    test) passes unchanged.
  - All of og-ui's tests pass (fields 12, widgets 27, style 16, text 8,
    gfx, kit_gfx, text_gfx, the rest).
- Row 58: the frames are a browser run, and no browser runs here. What
  runs without one:
  - `packages/engine/tests/text-fonts.test.ts` (bun, runs `og cook`).
    The harness's cook gives every font `text_scenes.rs` `FONTS` names,
    each in the manifest with its blob among the files handed to the
    module. It fails on the old code (no `tests/web/text-fonts.ts`, no
    `FONTS`). With a settings file removed it fails `Expected -1`.
  - The blobs the harness's `og cook` writes are byte-identical to those
    of `text_gfx`'s `cook_project` (all six `.ogb` compared with `cmp`).
  - `cargo check` and `cargo clippy -D warnings` of
    `-p og-tests-web --target wasm32-unknown-unknown` pass: the scenes,
    og-cook's font loader and the KTX2 decode compile for the browser.

## Frames read

- `text_gfx` before and after the move, on `vulkan-cpu` against `metal`:
  text-bidi RMSE 0.00065, text-kerning 0.00112, text-edit 0.00062,
  text-mixed-runs 0.00060, all with 0 outliers, the same figures both
  times. The four captured PNGs are byte-identical to those before the
  move (`cmp`). No reference was re-recorded: nothing in this stream may
  move a frame. Row 59's caret change draws the caret at its leading edge
  everywhere a visual move did not put it elsewhere, and no reference
  frame moves the caret across a bidi edge (text-edit's moves are Latin).
- Row 63 moves no frame: no `.vox` in the repository has a `META` chunk
  or `_loop` (checked: pivot.vox and Lantern Vale's six).

## References owed

- Browser (the coordinator, on the Mac): the four text frames' first runs
  on both tiers:
  `bun tests/web/reference-gfx.ts --scene og-ui/text-bidi,og-ui/text-kerning,og-ui/text-edit,og-ui/text-mixed-runs`,
  then the same with `--webgl`. They are judged against
  `crates/og-ui/references/metal/`. A per-tier copy should be recorded
  (`--record`) only where a tier differs by design. A plain
  `web-reference` run now includes them, because they are in
  `og_gfx_ui_scenes()`. The runner needs the `og` of
  `packages/engine/native/` (or `$OG_BIN`), which the web-reference lane
  has after build.sh.
- Metal: none (no frame moved).

## Shared files touched

- `Cargo.lock` (og-tests-web's three new dependencies)
- `crates/og-api/api.ogt` and its generated outputs: `index.d.ts`,
  `api.schema.json`, wiki `load.md`
- `docs/ci.md`
- `docs/phase-1.md` (the owed table)
- `packages/engine/tests/web/reference-gfx.ts`
- `packages/knowledge/skills/og-game/SKILL.md` and
  `packages/knowledge/skills/og-ui/SKILL.md` (the knowledge validate test
  passes, 21 of 21)
- `crates/og-ui/src/ui.rs` (`STATE_VERSION` 9)

## Left out

- Row 63: MagicaVoxel's frame rate is not in the file. `voxFps` stays
  the only source.
- Row 59: a press at a bidi edge still lands on the stop's leading place,
  and ↑/↓ keep that place's x (written in the og-ui README's owed list).
- Row 58: no browser run, no per-tier references (above). Missing glyphs
  are asserted natively only (`text_gfx`), not by the browser runner.

## Decisions for the coordinator

1. The brief named `reference-ui.ts`, but og-ui's Rust reference scenes
   ride `reference-gfx.ts` (`og-ui/<name>` in og-tests-web, where
   `text-fields-2` already runs). The text scenes went there, and
   `reference-ui.ts` (the kit's playtests) is unchanged. If they are wanted
   under reference-ui's report, that is a move of the runner, not of the
   scenes.
2. `reference-gfx.ts` now depends on an `og` binary (to cook) as well as
   cargo and wasm-bindgen; without one it stops after the module build with
   og-bin.ts's message (run `bun run build` or set OG_BIN).
3. `save_state` version 9 refuses version-8 bytes, as every bump has. No
   saved bytes persist across versions (world snapshots are in-process).
4. `CHANGELOG.md` is not touched (the coordinator's merge commit writes
   it).
