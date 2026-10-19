# Phase 2 terrain and water: break and restore

Every new test was broken once in the code it covers and failed, then the
code was restored and it passed. Each row is the edit made and the test
that failed with what it read (`cargo test`, 2026-10-19, Apple M4 Pro,
Metal). The breaks were applied by a script that edits one source file,
runs the named test and puts the file back (with a newer modification
time, so cargo rebuilds what it restored).

## og-render through og-tests (`crates/og-tests/tests/terrain.rs`, `crates/og-render/src/terrain.rs`)

| Broken | Edit | Failed |
|---|---|---|
| the clipmap's morph | `terrain.wgsl`: the morph never applies (`level.grid.z > 1e9`) | `the_clipmap_leaves_no_cracks`: 235 background pixels through the terrain at the first camera |
| where the levels stop | `levels()`: the coverage and far-plane break removed (always 16 levels) | `levels_follow_the_camera`: near 16 levels, high 16 |
| the ring's hole | `levels()`: the hole offset always (0, 0) | `levels_tile_the_plane_without_overlap_or_gaps`: cell 576,580 drawn 2 times |
| terrain casts | `terrain_casters` forced false (no terrain in the shadow passes) | `terrain_casts_and_receives_shadows`: the ridge's shadow 59.4 vs 59.4 without |
| terrain receives | `terrain_lit.wgsl`: the cascade lookup removed | the same test: 59.3 vs 59.4 |
| the slope rule | `layer_weights`: the slope band dropped | `layers_follow_slope_altitude_and_the_control_map`: the flat ground "mixed", not red |
| the control map | `layer_weights`: the control map's sample replaced by 1 | the same test: "mixed" where the control map paints green |
| edits | `sync`: an uploaded data is never uploaded again (revision ignored) | `an_edit_changes_the_drawn_surface`: the mound does not show (same renderer, next frame) |
| the CPU height's triangles | `TerrainData::height`: the upper triangle's slopes swapped | `the_cpu_height_and_normal_follow_the_field`: 3.66 at (0.3, 0.7) |

## Water (`crates/og-tests/tests/terrain.rs`)

| Broken | Edit | Failed |
|---|---|---|
| the planar reflection | `water.wgsl`: the planar branch never taken (`extra.y > 9.5`) | `the_planar_reflection_shows_what_the_screen_does_not`: planar [118, 143, 171] = screen-space |
| the reflection's clip plane | the mirrored camera's plain projection instead of the oblique one | the same test: the block under the water hides the post, [119, 141, 115] |
| shadows on water | `water.wgsl`: the shadow lookup never taken | `water_receives_shadows`: in the deck's shadow 228.6 vs 228.6 |
| the underwater fog | `underwater()` answers None | `the_underwater_view_fogs_shows_the_surface_and_caustics`: the post fogged from below [204, 49, 55] |
| Snell's window | `water.wgsl`: the window always closed | the same test: the window 114.0 over the fog 128.1 |
| caustics | `lighting.wgsl`: `caustics()` multiplied by 0 | the same test: caustics pattern 1.30 vs 1.30 |

## og-physics (`crates/og-physics/tests/heightfield.rs`)

| Broken | Edit | Failed |
|---|---|---|
| the cells' diagonal | `from_heightfield`: the zig-zag status not set (parry's other diagonal) | `heights_are_rows_along_z_split_along_the_renderers_diagonal`: the ray hits y 0 where the renderer's triangle is 1 m |
| rows and columns | `from_heightfield`: heights copied row-major into parry's column-major array | `the_surface_follows_the_field_and_its_edits` (-1.63 at 7.3, -2.2) and `a_character_walks_up_a_field_and_a_ball_rests_on_it` |
| edits reach the simulation | `set_heightfield`: the handle not marked stale | `the_surface_follows_the_field_and_its_edits`: the crate stays at -0.03 on the old field |
