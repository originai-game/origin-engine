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
