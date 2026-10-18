# E6 break and restore

Every new test was broken once in the code it covers and failed, then the
code was restored and it passed. Each row is the edit made, the test that
failed and the line it failed on (`cargo test`, 2026-10-19, Apple M4 Pro).

## og-nav (`crates/og-nav/tests/nav.rs`)

| Broken | Edit | Failed |
|---|---|---|
| jump point search's forced neighbours | `forced()` answers false for straight moves | `jump_point_search_finds_paths_as_short_as_astar_and_never_cuts_a_corner` (A* and JPS disagree on the cost) |
| no corner cutting | `can_step` allows a diagonal past one blocked orthogonal | the same test (the step cuts a blocked corner) |
| flow field directions | the direction picks a higher-cost neighbour | `a_flow_field_leads_every_reachable_cell_downhill_to_a_target` (goes uphill) |
| the funnel | portal sides swapped (left for right) | `a_baked_navmesh_routes_around_walls_up_ramps_and_stays_on_the_surface` (a corner inside the wall) |
| region rebuilds | an obstacle rebuilds every tile | `an_obstacle_rebuilds_only_its_tiles_and_removing_it_restores_them` |
| snapshots | the state block drops the agents' paths on restore | `two_hundred_agents_cross_a_256_grid_and_replay_identically_after_a_snapshot` (the restored world diverged) |
| separation | the push points at the neighbours | `separation_spreads_a_pile_of_agents_and_wander_is_deterministic` (agents still overlap) |
| walking the surface | `move_along` stops at the first wall instead of sliding | `an_agent_walks_a_navmesh_around_the_wall_and_arrives_on_the_surface` |

## og-ai (`crates/og-ai/tests/ai.rs`)

| Broken | Edit | Failed |
|---|---|---|
| reactive selectors | a selector keeps ticking its running child | `a_resource_fact_redirects_the_patrol_and_the_selector_is_reactive` (the patrol keeps running under the alarm) |
| command cooldowns | the cooldown is never checked | `a_guard_patrols_chases_on_sight_bites_on_a_cooldown_and_flees_when_hurt` (a bite every tick) |
| occlusion | sight rays are not cast | `walls_the_view_cone_and_range_hide_the_target_and_noises_are_heard_within_hearing` (seen through the wall) |
| the view cone | the cone test always passes | the same test (seen behind its back) |
| field conditions | field offsets one byte off | `a_resource_fact_redirects_the_patrol_and_the_selector_is_reactive` (`Game.alarm` misread) |
| machine timers | `after` is ignored | `a_state_machine_moves_on_facts_and_timers_and_runs_enter_and_exit` (left alert at once) |
| snapshots | memories dropped on restore | `two_runs_and_a_restored_snapshot_decide_the_same` |
| hearing | the hearing radius is ignored | `walls_the_view_cone_and_range_hide_the_target_and_noises_are_heard_within_hearing` (heard 9 m away) |

## og.nav and og.ai through the hosts (`packages/engine/tests/ai.test.ts`)

Each break rebuilt the dev addon (`OG_GATE=1 sh packages/engine/scripts/build.sh`) before the run.

| Broken | Edit | Failed |
|---|---|---|
| grid costs | `og.nav.grid` reads a 0 (blocked) cost as 1 | `og.nav: a grid agent routes round a wall …` (the path goes through the wall) and `og.nav: drive false …` (the remaining length) |
| navmesh boxes | `og.nav.build` drops every box (`hi.y > min.y` inverted) | `og.nav: a navmesh built from a floor and a box …` (the path does not go round) |
| `NavAgent.drive` | the drive check never skips the motion | `og.nav: drive false plans and reports but leaves the motion to the game` (the walker moved) |
| a speed read from a field | `speed Pace.chase` falls back to the agent's base speed | `og.ai: a guard on a behaviour tree …` (NavAgent.speed 2, not 5) |
| re-planning off the path | an agent far from its path's leg is not stale | `og.nav: an agent moved off its path … plans again` (no new plan) |

## The topdown slime on a behaviour tree (`templates/archetypes/topdown/tests/`)

| Broken | Edit | Failed |
|---|---|---|
| occlusion | the slime's `og.Perception` gets `occlude: false` | `flee` (the hero behind the pillar is seen and chased) |
| the bite | `biteHero` returns for any distance | `ai` (no hurt, no heart lost, no bite counted) and `combat` (the opening bite) |
| fear | `check Slime.hp below 0` in `assets/ai/slime.ogt` | `flee` (it chases at 20 hp and ends 1.70 m from the hero) |

## The 3D guard (`templates/game/tests/guard.og.ts`)

| Broken | Edit | Failed |
|---|---|---|
| occlusion | the guard's `og.Perception` gets `occlude: false` | `guard` (seen through the wall at once; searching and alarmed six times) |
| the cone before the first move | og.ai's forward falls back to zero instead of the Transform's facing | `guard` (the trickster 4 m behind it is seen; it then chases it the wrong way) |

## og-audio's E6 mixer and tick (`crates/og-audio/tests/mix.rs`)

| Broken | Edit | Failed |
|---|---|---|
| bus inserts | the bus low-pass is built as a high-pass | `bus_filters_shape_their_bus_only` |
| ducking | the ducker's target is always unity | `ducking_turns_music_down_under_effects_and_back` |
| the compressor | the gain computer never reduces | `the_compressor_reduces_a_loud_bus` |
| the reverb | the wet signal is multiplied by 0 | `a_reverb_send_leaves_a_tail` |
| scheduled starts | a start inside a tick gets no delay | `scheduled_starts_land_on_their_sample_and_stems_stay_aligned` (48000, not 48123) |
| the music clock | the next bar line rounds down | `the_music_clock_beats_and_a_stinger_lands_on_the_bar_line` |
| crossfades | a stop ignores its fade | `a_crossfade_ramps_over_its_fade` |
| the voice cap | the cap is never reached | `the_voice_cap_culls_the_lowest_priority` |
| distance low-pass | `distanceCutoff` is ignored | `occlusion_and_distance_dull_a_source` |
| occlusion gain | an occluded source keeps its gain | `occlusion_and_distance_dull_a_source` (low band 1.0, not 0.35) |
| listen | the ring records half of each render | `the_mix_is_the_same_twice_and_listen_returns_its_last_second` |

## Sound presets (`crates/og-cook/tests/sfx.rs`)

| Broken | Edit | Failed |
|---|---|---|
| synthesis | the output gain is 0 | `every_preset_renders_…`, `seed_pitch_and_length_vary_a_preset`, `a_cooked_preset_is_a_sound_the_runtime_loads` |
| pitch | `pitch` is ignored | `seed_pitch_and_length_vary_a_preset` (ratio 1, not 2) |
| refusals | an unknown preset passes `parse` | `malformed_presets_are_refused_with_the_fix`, `a_cooked_preset_is_a_sound_the_runtime_loads` |
| cooking | `type sfx` is not classified as a sound | `a_cooked_preset_is_a_sound_the_runtime_loads` |

## Audio through `og run` (`packages/engine/tests/audio.test.ts`)

| Broken | Edit | Failed |
|---|---|---|
| `og run --audio` | og-cli drops the flag | the test (mode `null`, not `capture`) |
| `listen` | the action's default is a quarter second | the test (12000 frames, not 48000) |

## og.platform (`packages/engine/tests/platform.test.ts`, `templates/archetypes/runner/tests/records.og.ts`)

| Broken | Edit | Failed |
|---|---|---|
| applying answers | og.platform drops each answer | all three platform tests and `records` |
| store conflicts | a write at a stale revision lands | `og.platform: a game's calls are answered …` (no `conflict`) |
| leaderboard around | the around query returns the top | the same test (ranks 1–3, not 12–14) |
| the runner's best | the crash does not save a new best | `records` |

## og-i18n (`crates/og-i18n/tests/i18n.rs`)

| Broken | Edit | Failed |
|---|---|---|
| Arabic plurals | `few` tests the integer, not its last two digits | `plurals_pick_each_languages_case` (103 is other, not few) |
| selects | no case matches its key | `messages_format_selects_numbers_quotes_and_missing_arguments` |
| refusals | a plural without `other` parses | `malformed_messages_and_tables_are_refused_with_where` |
| Chinese negotiation | every Chinese tag is Hans | `locales_negotiate_to_the_best_table` (zh-TW took zh-CN) |
| the fallback chain | the base table is never consulted | `the_catalog_falls_back_and_the_check_names_every_gap` |
| paragraph direction | no character is strong right to left | `directions_follow_the_first_strong_character_and_the_locale` |

## Shaping and right to left (`crates/og-ui/tests/rtl.rs`, `crates/og-cook/tests/strings.rs`, `packages/engine/tests/i18n.test.ts`)

| Broken | Edit | Failed |
|---|---|---|
| shaping | `Shaper::word` answers nothing | `arabic_words_are_shaped_into_joined_forms_in_visual_order` |
| right-to-left order | an RTL paragraph keeps its logical word order | `paragraphs_order_their_words_by_direction` |
| mirroring | right to left leaves the rectangles where they are | `a_right_to_left_ui_mirrors_its_layout` |
| the cook's key check | no table is checked | `tables_are_validated_and_every_locale_is_checked_against_the_base` |
| joined forms | the GSUB closure is the letters alone | `an_arabic_font_carries_its_joined_forms_and_its_file` |
| the host's locale | og_engine_create ignores `locale` | `og.i18n: the host's locale picks the table …` |
