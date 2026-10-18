# E3 break-and-restore

Each new test was run once against broken code (the break below), seen to
fail, and run again after the code was restored (the file was
copied aside, edited, the test run, the copy put back and touched so cargo
rebuilt it). No test was weakened.

| Break (file: change) | Test run | Result |
|---|---|---|
| og-ui/src/ui.rs `nine`: the centre piece skipped | `cargo test -p og-ui --test skin` | `a_skinned_panel_is_nine_pieces_of_the_atlas_that_tile_its_rectangle` FAIL (the pieces cover half the card) |
| og-ui/src/ui.rs `node_commands`: the skin slice lookup filtered out (always the token quad) | same | the same test FAIL |
| og-ui/src/skin.rs `parse_skin`: an unknown setting ignored | same | `a_skin_file_names_the_line_that_is_wrong` FAIL |
| og-ui/src/juice.rs `screen_transition`: `shown` switched as the cover starts | same | `a_screen_transition_covers_switches_at_full_cover_and_reveals` FAIL (switch at tick 0, not 11) |
| og-ui/src/juice.rs `counter`: the first count starts at the value, not `from` | same | `a_counter_counts_by_the_tick_and_replays_after_a_restore` FAIL |
| og-ui/src/ui.rs `button_motion`: the shake offset zeroed | same | `a_refused_press_shakes_the_button_and_queues_the_error_sound` FAIL (no movement) |
| templates/ui-kit/assets/skins/fantasy/skin.ogt: `fgMuted` lightened to `#a08560` | `cargo test -p og-ui --test packs` | FAIL: fantasy fgMuted on the dialog, screen panel, gallery panel and card under 4.5:1 |
| og-ui/src/ui.rs `dress`: panel tiles never drawn (dev addon rebuilt) | `og test templates/ui-kit` | `skin-fantasy`, `skin-scifi`, `skin-paper`, `skin-neon`, `skin-pixel` FAIL (their panels lose the texture); `skin-cartoon` (no tile) and every token suite PASS, as they must |
| og-api/src/runtime/modules.rs `og.ui.sound`: events taken and dropped (dev addon rebuilt) | `og test templates/ui-kit` | `juice` FAIL (no `og.AudioSource` voice after the open, hover and press) |
| packages/knowledge: `skills/og-taste` moved away | `bun test tests/validate.test.ts` | `the committed package is valid against the platform rules` FAIL (`skills/og-taste/SKILL.md: missing`, and the pack list) |

## The default skin is the token look, pixel for pixel

Before the skin layer (at b94d12d) `og test templates/ui-kit` and
`cargo test -p og-ui` were run and their frames kept: the 21
`frames-desktop` and 21 `frames-phone` captures and og-ui's
`sample-screen-metal.png` and `widget-set-metal.png`. After every og-ui
commit of this round the same runs were compared byte for byte on decoded
RGBA (PIL): 21 of 21, 21 of 21 and 2 of 2 identical, the last time at the
gallery commit (7db0908). The committed references
(`crates/og-ui/references/metal/kit-*.png`, `sample-screen.png`,
`widget-set.png`) are unchanged by the round (`git diff b94d12d --
crates/og-ui/references` lists only the six new `kit-skin-*.png`).
