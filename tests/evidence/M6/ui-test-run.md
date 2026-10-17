# og-ui M6 test run (4fb20c7 content, at 49ed935)

`CARGO_TARGET_DIR=target/agent-qa cargo test -p og-ui -- --nocapture` on Apple M4 Pro, Metal, 2026-10-18. All pass:

```
test a_scroll_container_clips_everything_it_draws ... ok
sprites 216 glyphs 173 sprite draws 4 levels 4 missing glyphs 0
metal: RMSE 0.00000, outliers 0.000% against /Volumes/app/origin-engine/crates/og-ui/references/metal/sample-screen.png
test sample_screen_matches_its_reference_frame ... ok
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.51s
test nested_rows_and_columns_lay_out_as_the_golden ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test a_scroll_container_scrolls_by_wheel_and_clips_its_content ... ok
test dump_ui_lists_every_visible_element_with_bounds_and_text ... ok
test move_to_text_resolves_exact_substring_and_nth_matches ... ok
test a_button_clicks_only_on_a_press_and_a_release_inside_it ... ok
test the_text_field_toggle_and_slider_edit_game_state ... ok
test restoring_the_world_and_the_ui_state_replays_the_same_frames ... ok
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
test theme_rs_is_generated_from_the_vendored_tokens ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test crates/og-ui/src/lib.rs - (line 6) ... ignored
```

## Frame read: `ui-sample-screen-metal.png`

The sample screen at 800×500 CSS px, dpr 1.5 (1200×750), dark scheme on the stage colour: a left menu panel ("Origin", the subtitle "A small game made with Origin Engine." wrapped on two lines, an amber primary **Start**, a neutral **Options**, **Quit** in the danger red, a switched-on amber **Music** toggle, an amber slider at about 60 %, an amber progress bar at about 40 %); a right **Store** panel with the CJK caption 原点引擎, "1250 coins" in amber, a focused search field showing "pot" with the caret and the amber focus ring, a scrolled list of rows "Item 1" … "Item 6" with neutral **Buy** buttons (a partly scrolled-off row cut at the top and bottom edges of the list, the scrollbar thumb at the right), the tooltip "120 coins" over Item 3's Buy button, and a wrapping tag row (Sword … Boots, Map on the second line). Matches `crates/og-ui/references/metal/sample-screen.png` with RMSE 0.00000; 216 sprites and 173 glyphs in 4 sprite draws over 4 painter levels, no missing glyph.

Note: the README (§Drawing) says "4 sprite draws (2 levels × atlas + font)"; the test reports **4 levels** for those 4 draws. The draw count matches; the level count in the README is stale or counts differently.

## `ui-dump-ui-sample.json`

Written by `dump_ui_lists_every_visible_element_with_bounds_and_text` (`tests/probe.rs`) through og-probe's command channel: `kind: ui`, tick 1, 47 elements in draw order with id, parent, text and CSS-pixel bounds cut to the clip (Screen, Menu, Title, Subtitle, Start, Options, Quit, Music, Volume, Loading, Store, Header, Heading, Engine, Coins, Search, List, Row#0…Row#6 each with Name and Buy, Tags, Tag#0…Tag#7). It is the probe test's own scene state at tick 1, not the state the sample frame shows (no tooltip yet).
