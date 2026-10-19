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

## og-cook (`crates/og-cook/tests/terrain.rs`)

| Broken | Edit | Failed |
|---|---|---|
| where tiles go | the runtime places a tile's samples with its rows and columns swapped | `a_terrain_cooks_into_tiles_and_loads_back`: (299, 0) -8.78 vs 86.27 |
| the heights' precision | the cook quantises to 8 bits | the same test: (299, 0) 86.08 vs 86.27 |
| the heightmap is the terrain's | the walk does not take the heightmap and control map out | the same test: `terrain/height.png` and `terrain/control.png` cooked as textures |
| the stamp | the terrain's stamp leaves out its heightmap | `the_terrain_cook_is_deterministic_and_incremental`: the edited heightmap is not cooked again |
| EXR heights | an EXR's default range 0 to 100 like a PNG's | `an_exr_heightmap_is_metres_as_stored`: (0, 0) 12.46 vs 0.125 |
| a terrain in a navmesh | the terrain's triangles wound clockwise (facing down) | `a_navmesh_bakes_a_terrains_walkable_surface`: nothing walkable at x -30 |

## og.terrain through the native host (`packages/engine/tests/terrain.test.ts`)

Each edit made to `crates/og-runtime/src/terrain.rs`, the addon rebuilt (`sh scripts/build.sh`), the file run, the edit undone and the addon rebuilt (5 of 5 pass).

| Broken | Edit | Failed |
|---|---|---|
| sample's world height | `og_terrain_sample` answers the height without the origin's y | `og.terrain.load answers the terrain, and sample its surface` |
| the collider at load | `load` registers no height field with physics | every test: the og.Collider of shape SHAPE_HEIGHTFIELD names an unknown field, so the first tick is refused (including `a ball rests on the terrain and a character walks up the hill`) |
| edits reach physics | `og_terrain_edit` does not register the edited heights | `an edit raises the ground for sample, physics and the renderer`: the ray meets the old ground |
| edits reach the renderer | `og_terrain_edit` does not call `set_terrain` | the same test: the frame does not change around the mound |
| setLayer | `og_terrain_set_layer` keeps the layer's old colour | `og.Terrain draws the terrain, and a layer reads back and retunes` |
| a brush that misses | the bounds check before an edit removed | `og.terrain refusals name the fix`: a brush 500 m away is accepted |

The character test was first written expecting x > -12 after 240 ticks at 3 m/s from -24; the climb costs it about a metre, so it now requires x > -15 and the ground under it 2 m higher than at the start (the walk up the flank), not a distance that only a flat run reaches.

## fx-showcase's terrain station (`templates/fx-showcase/tests/terrain.og.ts`)

| Broken | Edit | Failed |
|---|---|---|
| the hero stands on the terrain | `src/p2.ts`: the valley's SHAPE_HEIGHTFIELD collider not inserted | `it stands on the terrain`, `its feet are on the sampled ground (y -111.77, ground 7.98)`: it fell through |
