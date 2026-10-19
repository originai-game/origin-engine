# Phase 2, UI round: break and restore

Each new test of the round was run once against the code it covers broken
on purpose, seen to fail for the right reason, and run again after the code
was restored.

| Test | Broken how | Failed with | Restored |
|---|---|---|---|
| `packages/engine/tests/ui-sound.test.ts` (row 15) | `crates/og-runtime/src/modules.rs` as on main: `og.ui.sound` scheduled after `og_audio::install` | `expect(received).toBeLessThan(expected)`: the hover blip's first frame 72800, expected < 72800 (its input's tick is frames 72000..72800; the blip started one tick late) | passes, first frames 72000 and 120000 |
| `crates/og-ui/tests/skin.rs` `keyboard_and_gamepad_focus_moves_sound_as_the_pointer_does` (row 41) | the `Hover` sound of a focus move in `crates/og-ui/src/nav.rs` removed | `Tab onto A sounds`: left `[]`, right `[Hover]` | passes |
| `crates/og-ui/tests/skin.rs` `the_hover_lift_and_the_press_move_on_springs` (row 16) | `button_motion`'s spring step replaced by the target (`a.a = target`: the lift jumps) | `moving over several ticks, not at once`: every top edge 45.68 from the first tick | passes; `og test templates/ui-kit` passes all 25 suites with the six packs' desktop and phone sheets unchanged (each hovered or pressed control in them is captured at rest, where the spring ends exactly where the eased curve did) |
| `templates/archetypes/runner/tests/flow.og.ts`, the held press's sound (row 46) | `ask(world, Sound.Hold)` removed from Director | `but it is answered by the hold tick (sfx-16: Sfx, og.AudioSource, og.Transform)`: no og.AudioPlayback | passes; the runner's 8 suites pass |
| `templates/ui-kit/tests/refuse.og.ts` (row 45) | `kit.shake({ sound: true })` removed from the shop's refusing Buy (`packages/engine/src/ui-kit/store.ts`) | `the button shook sideways after the press (up to 0.00 px …)` and `the error sound plays on the ui bus (voices [])` | passes; the kit's 26 suites pass |

Rows 42 (type sizes) changed no test: `og polish` is its measure (the
reports in `polish/`), and the 16 kit references it moved were re-recorded
(`rerecord/row42-*.png`, old left, new right).
| `crates/og-ui/tests/gfx.rs` `a_text_glow_is_a_halo_from_the_distance_field` (row 18) | the glow's radius no longer capped at the field's reach in `overlay.wgsl` | `nothing glows past the distance field's reach`: left 6, right 0 (the halo ended at the glyph quad's edge) | passes. Swapping the passes (fills before glows) was also tried: with the test font's 2-pixel reach the glows of neighbours never reach ink, so it passed; the order is kept for wider fields and stated, not proven, by this test |
| row 17 (tracking) | no new test: the token ramp's tracking moved 41 kit references and `sample-screen.png`, re-recorded with before and after (`rerecord/row17-*.png`) | the old references fail at 0.04 to 0.53 % outliers, RMSE 0.010 to 0.048 | the kit's 26 suites and og-ui's tests pass |
| `crates/og-ui/tests/widgets.rs` `a_steps_tracking_spaces_latin_glyphs_and_not_cjk` (row 17) | `step_tracking` answering 0 | `six glyphs, each 0.1 em further`: equal widths | passes |
| `crates/og-ui/tests/widgets.rs` `pads_beyond_the_first_move_their_own_focus_and_press` (row 21) | `Ui::begin` no longer calling `apply_pads` | `focus_path(1)`: left `""`, right `"Lobby/R#2/B21"` (originally written against the first-control start; re-run against the centre start) | passes; the kit's `players` playtest passes with its new reference `kit-desktop-players.png` |
| `crates/og-ui/tests/widgets.rs` `scroll_views_scroll_both_ways_hand_off_fling_and_grab` (row 33, scrolling) | (1) the handoff stopped after the innermost container; (2) a released drag starting no fling | (1) `what the strip cannot take scrolls the page`; (2) `released while moving, it keeps going`: the scrollbar still | passes; the kit's 27 suites pass |
| `crates/og-ui/tests/rich.rs` `spans_keep_their_style_and_links_are_elements` (row 33, rich text) | `rich_commands` drawing every span at weight 0 | `[b] thickens its glyphs only` | passes |
| `crates/og-ui/tests/rich.rs` `lines_break_between_words_and_keep_a_restyled_word_whole` | units of a word split across spans no longer glued | `un\|break\|able stays on one line` | passes |
| `crates/og-ui/tests/rich.rs` `a_link_answers_a_click_and_keyboard_focus` | link keys left out of `end()`'s present map (the press is forgotten before release) | `a click answers the link's id`: left `None`, right `Some("help")` | passes |
| `crates/og-ui/tests/rich.rs` `per_glyph_motion_moves_by_the_tick_and_rests_under_reduced_motion` | moving spans drawn whole | `a moving span is drawn glyph by glyph`: left 0, right 4 | passes |
| `crates/og-ui/tests/rich.rs` `markup_it_cannot_read_shows_as_text_with_a_warning` | an unknown tag dropped instead of kept as text | `the unknown tag stays visible`: left `"a b[/blink] c"` | passes |
| `crates/og-ui/tests/rich.rs` `a_right_to_left_paragraph_runs_its_spans_from_the_right` | punctuation-only pieces of a right-to-left line drawn in logical order | `no piece " ."` (the stop drawn as `". "`, its space between the bold digit and the stop, as the Arabic confirm frame first showed) | passes |
| `crates/og-ui/tests/gfx.rs` `bold_rich_text_is_heavier_ink_never_a_box` | (1) `overlay.wgsl` no longer capping the weight at the field's reach; (2) bold's second copy removed | (1) `bold ink at the reading size stays on its glyphs`: 2 stray pixels (the kit's fonts, with a 4 px range, drew a filled box behind the bold digit: the first capture of `kit-desktop-confirm.png`); (2) `bold is heavier at the xs size: 247 against 247 px` | passes |
| `templates/ui-kit/tests/players.og.ts`, the hint's link | `og.ui.richText` in `packages/engine/src/runtime.ts` answering `link: null` | the tap on `Hint/join` leaves the demo's `rules` at 0 (`og run` with the playtest's taps; the check `the tap on the hint's How to join link followed it once` reads it) | `rules` 1; the kit's 27 suites pass |
