# UI shell round: each new behaviour broken once

Each break is one edit to the code the test covers, the test run, and the
file put back (byte for byte; the suites pass again). 2026-10-21, Apple M4
Pro, metal, `cargo test` in the worktree.

| Behaviour | Break | Test | What failed |
|---|---|---|---|
| A screen reader's requests reach the UI | `begin_with` no longer calls `apply_a11y` | `tests/a11y.rs` `a_screen_readers_requests_reach_the_control_the_tree_names` | focused `""`, expected `Form/Hints` |
| The page's reduced motion reaches the theme | `effective_theme` drops `t.reduced_motion \|= viewport.reduced_motion` | `tests/a11y.rs` `the_pages_motion_and_contrast_preferences_reach_the_theme` | `reduced motion true`: left false |
| The edge light | `has_edge` never taken | `tests/theme.rs` `a_dark_card_draws_the_edge_light_inside_its_edge` | one quad (the fill) where three are drawn |
| Icon sets draw glyphs | `IconAtlas::glyph` answers none | `tests/widgets.rs` `an_icon_set_draws_its_glyphs_in_their_squares` | no icon glyph drawn (0, expected 2) |
| Aspect ratios | the fitting height ignores `aspect` | `tests/widgets.rs` `two_fingers_scroll_lists_and_pinch_zoom_views` | the 2:1 image is not 400 x 200 |
| Two touches | `scroll_input` never starts a gesture | same | the list moved 36 px, not 60 (the first finger's drag alone, with the row pressed) |
| The a11y ring slot | the decoder no longer knows kind 5 | `og-input` `tests/ring.rs` (both tests) | the two a11y events are missing from the drained events |
| Butt caps in the icon cook | the cap cut skipped | `og-cook` `cook::icons` `a_round_stroke_is_its_width_and_a_butt_end_is_cut` | 1.0 units past a butt end where 2.0 is right |
| Glass over overlay layers | `over_content` forced false (the old one-pyramid path) | `tests/gfx.rs` `a_glass_panel_blurs_the_overlay_layers_below_it` | `the bar under the glass spreads (150, 84): [187, 187, 187]` (grey stripes: the bar was not in the pyramid) |
| The pack palettes | the old palettes (before 7214094) | `tests/pack_contrast.rs` | 31 rows under their bar (pack-contrast-table.txt has the new rows) |
| Two touches in the kit | `touch:0:...` for the primary finger (refused by the probe) and the gesture without the same-scroller rule | `templates/ui-kit/tests/gestures.og.ts`, `multitouch.og.ts` | the list did not move; the stick lost its deflection while a second finger tapped Hit (the rule that two touches are a gesture only on one scroller came from this) |
