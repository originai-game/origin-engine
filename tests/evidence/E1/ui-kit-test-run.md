# UI kit round (E1): test run, 2026-10-18, Apple M4 Pro, Metal

Shared tree at 3db4319 plus this round's uncommitted changes. Concurrent
agents' half-finished edits broke the shared build at times (og-render and
og-api's anim wiring), so the TypeScript and host checks ran on an isolated
snapshot: `git archive HEAD` plus only this round's files (crates/og-ui,
the IDL `ui` module spliced into HEAD's api.ogt, og-api runtime/modules.rs
and the ui exports, the og_ui_* host impls, packages/engine/src/ui-kit,
packages/engine/src/index.ts's two lines, templates/ui-kit), regenerated with
`og api gen`.

## Rust (shared tree, CARGO_TARGET_DIR=target/agent-uikit)

```
cargo test -p og-ui
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test a_scroll_container_clips_everything_it_draws ... ok
test sample_screen_matches_its_reference_frame ... ok
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.46s
test the_widget_set_matches_its_reference_frame ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.09s
test nested_rows_and_columns_lay_out_as_the_golden ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test dump_ui_lists_every_visible_element_with_bounds_and_text ... ok
test a_scroll_container_scrolls_by_wheel_and_clips_its_content ... ok
test move_to_text_resolves_exact_substring_and_nth_matches ... ok
test the_text_field_toggle_and_slider_edit_game_state ... ok
test a_button_clicks_only_on_a_press_and_a_release_inside_it ... ok
test restoring_the_world_and_the_ui_state_replays_the_same_frames ... ok
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
test theme_rs_is_generated_from_the_vendored_tokens ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test a_game_accent_derives_its_tones_the_way_the_tokens_derive_amber ... ok
test a_transition_advances_by_tick_takes_no_input_while_moving_and_leaves ... ok
test arrows_move_to_the_aligned_neighbour_and_wrap_at_the_edges ... ok
test anchors_sit_in_the_safe_area_and_touch_controls_keep_44px ... ok
test the_gamepad_dpad_and_stick_navigate_with_repeat_by_tick ... ok
test the_dialogue_types_by_tick_a_press_finishes_and_the_next_press_advances ... ok
test a_select_opens_chooses_and_closes_on_a_press_outside ... ok
test a_long_list_builds_only_the_rows_in_view_and_scrolls_focus_into_view ... ok
test tab_walks_reading_order_and_shift_tab_walks_back ... ok
test a_dialog_traps_focus_blocks_what_is_under_it_and_closes_on_back ... ok
test a_press_dragged_from_one_cell_to_another_drops_there_once ... ok
test toasts_stack_three_leave_on_time_and_replay_after_a_restore ... ok
test a_meter_trail_follows_a_drop_by_tick_and_replays_identically ... ok
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
test crates/og-ui/src/lib.rs - (line 6) ... ignored
test result: ok. 0 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

- `cargo clippy -p og-ui --all-targets -- -D warnings`: clean (shared tree);
  `cargo clippy -p og-ui -p og-api -p og-host-node --all-targets --features og-api/runtime -- -D warnings`
  and `cargo clippy --target wasm32-unknown-unknown -p og-host-web -p og-ui -- -D warnings`: clean (snapshot).
- `cargo fmt --check -p og-ui -p og-api -p og-host-node -p og-host-web`: clean (snapshot).
- `og api check`: "the generated files match crates/og-api/api.ogt" (shared tree and snapshot).
- `crates/og-ui/scripts/gen-theme.sh --offline`: theme.rs unchanged; tests/theme.rs passes.

## TypeScript (snapshot)

`bun run test` in packages/engine (typecheck of the declarations and the
runtime including src/ui-kit, both hosts built, bun tests): 24 pass, 0 fail.
`bun scripts/validate.ts` in packages/knowledge: valid, skill og-ui included.

## The demo's playtests (snapshot, `og test templates/ui-kit`, native host)

```
PASS drag (4 checks, 139 ms): a drag from one inventory slot to another moves the item
PASS frames-desktop (45 checks, 945 ms): every screen at desktop size matches its reference frame
PASS frames-phone (45 checks, 5770 ms): every screen on a phone in portrait matches its reference frame
PASS keys (5 checks, 471 ms): keyboard focus opens a screen, steps a slider and closes the settings with a save
PASS pad (5 checks, 528 ms): gamepad navigation moves through the gallery, opens a screen and backs out
PASS phone (10 checks, 375 ms): on a phone the HUD keeps to the safe area and its touch controls to 44 px
PASS touch (10 checks, 1095 ms): touch_move_to and move_to_text drive the gallery, the shop, results, idle and the cutscene
PASS tutorial (5 checks, 345 ms): the tutorial spotlights each target and Next walks to the end
og test: 8 suite(s) passed
```

Every frame of frames-desktop and frames-phone compared with RMSE 0.00000,
0.000 % outliers against crates/og-ui/references/metal/kit-*.png (recorded
with OG_UI_CAPTURE=1, then re-run from a clean output: identical).

## Browser (snapshot, `og build templates/ui-kit`, headless Chrome for Testing 150, WebGPU, cross-origin isolated)

The M6 web procedure (tests/evidence/M6/games-web-procedure.md, drive.ts,
WINDOW=1280,863) with the script
`0.5:show:settings;1.2:key_down:ArrowDown;…;2.0:show:shop;3.0:touch_move_to:Detail/Buy;3.05:touch_start;3.1:touch_end`:
facts at tick 600 `{screen: shop, balance: 950}` (the Buy went through), world
hash 4413227e518de273 — equal to the native host's `og run --headless` of the
same script at tick 600 (4413227e518de273). Frame: ui-kit-web-shop.png.

## Frames read

- ui-kit-desktop-sheet.png: all 21 screens at 1280×720 — gallery list;
  widget sheet (tabs, select, stepper, radio, checkboxes, chips and badges,
  health/mana/xp meters, three cooldown rings, mini-map, spinner, skeleton,
  key-value rows, icons; stick, d-pad, Hit); title card over the dusk
  backdrop; pause dialog over the platformer HUD; settings with Audio tabs and
  sliders; platformer HUD (hearts, level, timer, coins, score, pause); shooter
  (health/armor meters, mini-map, abilities as rings, ammo 18/120, crosshair);
  puzzle (score 680 toward 1,000 with two of three stars, moves, timer); idle
  (gold and gems, Tap, generators list with Buy prices); racing (3/8, lap 2/3,
  time, best, mini-map, 234 km/h, gear 4, boost); cutscene (letterbox,
  portrait, typed line, Skip); shop; inventory; level select with stars and
  locks; results (two stars, 12,840, New best, stats, Retry/Next/Menu, Share);
  leaderboard (scopes, ranks with medal ink); tutorial (spotlight on Play with
  its callout); loading (title, real bar, tip); confirm (danger Delete);
  alert; toasts (stack at the bottom).
- ui-kit-phone-sheet.png: the same at 390×844@3 with a 47 px notch and 34 px
  home indicator: corners below the notch and above the indicator, settings
  tabs become a select, the shop and inventory stack their detail panel under
  the grid, HUDs show touch controls (d-pad and Jump; stick and Fire; Left,
  Right, Brake, Boost), the idle generators move to the bottom.
- ui-widget-set-metal.png: og-ui's own widget frame (tests/kit_gfx.rs).

Observed (not this round's code): sprites submitted with og.render.sprite on
one tick appeared to stay on later ticks that submitted none (the demo now
submits its backdrop every tick on the screens that show it).
