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
