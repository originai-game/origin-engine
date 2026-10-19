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
