# E5 break-and-restore

Each new test was run once against broken code (the break below), seen to
fail, and run again after the code was restored. A script applied each
break, ran the test and restored the file in a `finally`, so no break was
left in place and no test was weakened. Rust tests ran through `cargo
nextest`. TypeScript and template breaks rebuilt the dev native addon and
ran `bun test tests/e5.test.ts` or `og test templates/fx-showcase` (the
station named in the Test column must fail).

| Break | File | Test | Result |
|---|---|---|---|
| hinge angle never computed (value stays 0) | `crates/og-physics/src/dim3.rs` | a_hinge_swings_about_its_world_anchor_and_reports_its_angle | FAILED (as it must): Joint { kind: 1, body: 0, other: 0, anchor: Vec3(-1.0, 0.0, 0.0), other_anchor: Vec3(0.0, 5.0, 0.0), axis: Vec3(0.0, 0.0, 1.0), limits: false, min: 0.0, max: 0. |
| joint limits not passed to rapier | `crates/og-physics/src/ext.rs` | a_hinge_with_limits_stops_at_them | FAILED (as it must): angle -0.48630482 past the 0.4 rad limit |
| velocity motor ignored | `crates/og-physics/src/ext.rs` | a_velocity_motor_spins_a_wheel_and_a_position_motor_turns_to_its_target | FAILED (as it must): spinning at [0, 0, 0] |
| fixed joint does not keep the offset (anchors only) | `crates/og-physics/src/ext.rs` | a_fixed_joint_keeps_two_bodies_as_one | FAILED (as it must): [1.0530992, 0.5999999, 0.5753107] became [0, 0, 0] |
| slider locks every axis | `crates/og-physics/src/ext.rs` | a_slider_moves_along_its_axis_within_its_limits | FAILED (as it must): slid 0 |
| spring stiffness dropped | `crates/og-physics/src/ext.rs` | a_spring_settles_below_its_rest_length_and_a_rope_holds_at_its_length | FAILED (as it must): the spring hangs 2.9612 |
| a joint is kept when its body was rebuilt (stale handle) | `crates/og-physics/src/ext.rs` | a_joint_survives_its_body_being_rebuilt_and_stops_when_removed | PASSED: this break changed no behaviour. The joint's two handles are compared too, and a rebuilt body has a new handle, so the dropped `alive` check was a second guard. It was replaced by the next row, which drops every check but the structure |
| a joint to a non-body goes to the world anchor | `crates/og-physics/src/ext.rs` | a_joint_to_an_entity_that_is_not_a_body_does_nothing_until_it_is | FAILED (as it must): no body, no joint: the bob fell to -1 |
| 2D hinge angle never computed | `crates/og-physics/src/dim2.rs` | the_2d_profile_hinges_in_the_plane | FAILED (as it must): assertion failed: joint(&w, bob).value.abs() < 1.08 |
| character jumps every tick jump is held | `crates/og-physics/src/ext.rs` | a_character_walks_at_its_speed_stays_on_the_ground_and_jumps_once_per_press | FAILED (as it must): assertion `left == right` failed: holding jump does not jump again |
| character steps disabled | `crates/og-physics/src/ext.rs` | a_character_climbs_gentle_slopes_and_steps_but_not_steep_ones_or_tall_steps | FAILED (as it must): stepped up onto the low block: [1.6165116, 0.9214952, -0.9998117] |
| character slope limit ignored | `crates/og-physics/src/ext.rs` | a_character_climbs_gentle_slopes_and_steps_but_not_steep_ones_or_tall_steps | FAILED (as it must): did not climb the 60 degree ramp: [-8.075262, 4.0678496, -0.000113081696] |
| character not carried by its platform | `crates/og-physics/src/ext.rs` | a_character_rides_a_moving_platform_and_pushes_crates | FAILED (as it must): carried [0, -0.3023578, 0] |
| character pushes nothing | `crates/og-physics/src/ext.rs` | a_character_rides_a_moving_platform_and_pushes_crates | FAILED (as it must): the crate was pushed to [2, 0.5, 0] |
| platformer coyote time lost | `crates/og-physics/src/platformer.rs` | coyote_time_and_the_jump_buffer_forgive_late_and_early_presses | FAILED (as it must): a press 4 ticks after the edge jumps |
| platformer jump buffer lost | `crates/og-physics/src/platformer.rs` | coyote_time_and_the_jump_buffer_forgive_late_and_early_presses | FAILED (as it must): the buffered press jumped on landing |
| platformer jump cut disabled (no variable height) | `crates/og-physics/src/platformer.rs` | a_platformer_jumps_to_its_jump_height_in_its_time_to_apex_and_a_tap_jumps_lower | FAILED (as it must): a tap rose 3.3329833 against 3.3488872 |
| one-way platforms block from below | `crates/og-physics/src/platformer.rs` | one_way_platforms_are_solid_from_above_and_down_drops_through | FAILED (as it must): jumped up through and stands on it at 0.8071006 |
| drop-through never starts | `crates/og-physics/src/platformer.rs` | one_way_platforms_are_solid_from_above_and_down_drops_through | FAILED (as it must): dropped through to 2.9101 |
| wall slide speed not limited | `crates/og-physics/src/platformer.rs` | a_wall_slows_the_fall_and_a_wall_jump_pushes_away | FAILED (as it must): sliding at -4.2666664 |
| wall jump pushes towards the wall | `crates/og-physics/src/platformer.rs` | a_wall_slows_the_fall_and_a_wall_jump_pushes_away | FAILED (as it must): pushed off the wall to [0.09162844, 8.265878, 0] |
| corner correction never nudges | `crates/og-physics/src/platformer.rs` | corner_correction_slides_a_jump_past_a_ceiling_corner | FAILED (as it must): with correction it slid past, up to 1.9399 |
| platformer not carried by its platform | `crates/og-physics/src/platformer.rs` | a_platformer_rides_a_moving_platform | FAILED (as it must): carried to [0, 0.8101, 0] |
| 3D car steers the wrong way | `crates/og-physics/src/vehicle3.rs` | a_3d_car_rests_on_its_wheels_drives_forward_steers_and_brakes | FAILED (as it must): turned right to [6.2543063, 0.8748185, 12.496108] |
| 2D car never drives | `crates/og-physics/src/vehicle2.rs` | a_2d_car_drives_on_sprung_wheels_and_its_wheels_go_with_it | FAILED (as it must): drove right to -0.00022431176 |
| 2D wheels left behind when the vehicle goes | `crates/og-physics/src/vehicle2.rs` | a_2d_car_drives_on_sprung_wheels_and_its_wheels_go_with_it | FAILED (as it must): on its wheels at 0.54243183, without them it sat down to 0.54240257 |
| rope links not held | `crates/og-physics/src/verlet.rs` | a_rope_hangs_at_its_length_follows_its_anchor_and_drapes_over_a_box | FAILED (as it must): the rope is 6.0419297 long |
| rope start does not follow its entity | `crates/og-physics/src/ext.rs` | a_rope_hangs_at_its_length_follows_its_anchor_and_drapes_over_a_box | FAILED (as it must): assertion failed: (pts[0] - Vec3::new(2.0, 6.0, 0.0)).length() < 1e-4 |
| cloth ignores the wind on its normal | `crates/og-physics/src/verlet.rs` | cloth_hangs_from_its_pins_flutters_in_the_wind_and_drapes_over_a_ball | FAILED (as it must): the wind lifted it to z 0 |
| fluid ignores solids | `crates/og-physics/src/fluid.rs` | a_2d_fluid_settles_into_its_container_without_leaking_or_blowing_up | FAILED (as it must): leaked: [-3.4176178, -56.563164] |
| fluid has no pressure | `crates/og-physics/src/fluid.rs` | a_2d_fluid_settles_into_its_container_without_leaking_or_blowing_up | FAILED (as it must): spread to a mean x of -0.36958885 |
| 3D water: crossings measured against the dented surface | `crates/og-physics/src/water.rs` | a_body_dropped_in_water_splashes_makes_waves_that_die_out_and_floats | FAILED (as it must): the light crate floats at 1.8010212 |
| 3D water: no buoyancy | `crates/og-physics/src/water.rs` | a_body_dropped_in_water_splashes_makes_waves_that_die_out_and_floats | FAILED (as it must): the light crate floats at -2.5000498 |
| 3D water: waves never damped | `crates/og-physics/src/water.rs` | a_body_dropped_in_water_splashes_makes_waves_that_die_out_and_floats | FAILED (as it must): the waves died down from 0.8004808 to 1.533251 |
| 2D water line: crossings do not kick the springs | `crates/og-physics/src/water.rs` | a_2d_water_line_is_splashed_by_bodies_and_characters_and_settles | FAILED (as it must): the surface moved by 0 |
| ragdoll bodies ignore the animation targets | `crates/og-physics/src/ragdoll3.rs` | a_ragdoll_follows_its_animation_goes_limp_and_recovers_with_a_blend | FAILED (as it must): bone at [0, 1, 0] for target [0.0033333336, 1, 0] |
| ragdoll never goes limp | `crates/og-physics/src/ragdoll3.rs` | a_ragdoll_follows_its_animation_goes_limp_and_recovers_with_a_blend | FAILED (as it must): the head fell to [0.174, 1.8, 0] |
| ragdoll blend jumps instead of ramping | `crates/og-physics/src/ragdoll3.rs` | a_ragdoll_follows_its_animation_goes_limp_and_recovers_with_a_blend | FAILED (as it must): one tick into the blend: 1 |
| rope state outside the snapshot (rebuilt each tick) | `crates/og-physics/src/ext.rs` | physics_beyond_rigid_bodies_is_deterministic_and_restores_mid_run | PASSED: this break changed no behaviour across the restore. The rope was rebuilt on ticks that are multiples of 600 in both runs, which is still deterministic. It was replaced by `ropes outside the snapshot (not saved)` |
| fluid neighbours in hash order (unsorted grid) | `crates/og-physics/src/fluid.rs` | physics_beyond_rigid_bodies_is_deterministic_and_restores_mid_run | FAILED (as it must): |
| a joint is kept when its bodies changed (structure compared only) | `crates/og-physics/src/ext.rs` | a_joint_survives_its_body_being_rebuilt_and_stops_when_removed | FAILED (as it must): after the rebuild the arm is 3.3744743 |
| fluid has no near pressure | `crates/og-physics/src/fluid.rs` | a_2d_fluid_settles_into_its_container_without_leaking_or_blowing_up | FAILED (as it must): spread to a mean x of -0.6743761 |
| ropes outside the snapshot (not saved) | `crates/og-physics/src/sim.rs` | physics_beyond_rigid_bodies_is_deterministic_and_restores_mid_run | FAILED (as it must): assertion `left == right` failed: rope e11v1 |
| controller timers outside the components (coyote kept in a static) | `crates/og-physics/src/platformer.rs` | physics_beyond_rigid_bodies_is_deterministic_and_restores_mid_run | FAILED (as it must): assertion `left == right` failed: two 2D runs diverged |
| fluid particles outside the snapshot (not saved) | `crates/og-physics/src/sim.rs` | physics_beyond_rigid_bodies_is_deterministic_and_restores_mid_run | FAILED (as it must): assertion `left == right` failed: fluid e9v1 |
| fog never mixed in | `crates/og-render/shaders/frame.wgsl` | fog_hides_the_distance_not_the_foreground | FAILED (as it must): far detail 34.4 -> 34.6 with fog |
| no stars at night | `crates/og-render/shaders/frame.wgsl` | day_night_darkens_towards_night_with_stars | FAILED (as it must): night has stars (0) |
| day-night light does not fall at night | `crates/og-render/src/atmosphere.rs` | day_night_darkens_towards_night_with_stars | FAILED (as it must): 3 vs 3 |
| water colour ignores depth | `crates/og-render/shaders/water.wgsl` | water_darkens_with_depth_foams_ripples_and_reflects | FAILED (as it must): shallow 148.5 vs deep 159.8 |
| no foam | `crates/og-render/shaders/water.wgsl` | water_darkens_with_depth_foams_ripples_and_reflects | FAILED (as it must): foam lights a band where water meets geometry (0 pixels) |
| height field ignored | `crates/og-render/shaders/water.wgsl` | water_darkens_with_depth_foams_ripples_and_reflects | FAILED (as it must): the height field moves the surface (0.00) |
| 2D water ignores its surface | `crates/og-render/src/water.rs` | water_2d_follows_its_surface_in_the_sprite_order | FAILED (as it must): the lip rose by 2 units: 418 -> 418 |
| 2D water drawn after every sprite (top layer) | `crates/og-render/src/water.rs` | water_2d_follows_its_surface_in_the_sprite_order | FAILED (as it must): above the water: pure red [138.0, 136.0, 169.0] |
| metaballs without the threshold | `crates/og-render/shaders/metaballs.wgsl` | metaballs_merge_within_reach | FAILED (as it must): apart: 255 |
| surface_color hook never called | `crates/og-render/shaders/pbr.wgsl` | shader_hooks_dissolve_and_sway | FAILED (as it must): amount 0.6 cuts holes: 40000 -> 40000 |
| vertex_offset hook never applied | `crates/og-render/shaders/pbr.wgsl` | shader_hooks_dissolve_and_sway | FAILED (as it must): the grass moved (0.00) |
| shader parameters never reach the uniform | `crates/og-render/src/lib.rs` | shader_hooks_dissolve_and_sway | FAILED (as it must): amount 0.6 cuts holes: 40000 -> 40000 |
| cook skips naga compilation | `crates/og-cook/src/shader.rs` | shaders_that_do_not_compile_are_refused_with_the_reason | FAILED (as it must): no error for s/syntax.ogt: ["assets/s/kind.ogt: line 6: parameter type `matrix` (float, vec2, vec3, vec4 or color)", "assets/s/missing.ogt: line 4: source \"sha |
| cooked text loses the parameters | `crates/og-cook/src/shader.rs` | a_shader_cooks_for_every_back_end_and_loads | FAILED (as it must): assertion `left == right` failed |
| cook errors keep composed-shader line numbers | `crates/og-cook/src/shader.rs` | shaders_that_do_not_compile_are_refused_with_the_reason | FAILED (as it must): the location is a line of the hooks: assets/s/syntax.ogt: the shader does not compile: WGSL (full): error: expected expression, found ";" |
| weather ignores dim | `crates/og-render/src/weather.rs` | weather_presets_match_their_references | FAILED (as it must): assertion `left == right` failed |
| 2D weather drawn as billboards | `crates/og-render/src/weather.rs` | weather_presets_match_their_references | FAILED (as it must): assertion `left == right` failed |
| rope readout refuses every entity | `crates/og-runtime/src/physics_ext.rs` | bun tests/e5.test.ts | FAILED (as it must): error: entity 4294967300 has no og.Rope (or it has not stepped yet) |
| waterHeight ignores the height field | `crates/og-runtime/src/physics_ext.rs` | bun tests/e5.test.ts | FAILED (as it must): error: expect(received).toBeTrue() |
| timeOfDay inside a system ignores the clock | `crates/og-runtime/src/atmosphere.rs` | bun tests/e5.test.ts | FAILED (as it must): error: expect(received).toBeCloseTo(expected, precision) |
| day-night clock ignores speed | `crates/og-runtime/src/atmosphere.rs` | bun tests/e5.test.ts | FAILED (as it must): error: expect(received).toBeCloseTo(expected, precision) |
| weather particles never placed | `crates/og-runtime/src/atmosphere.rs` | og weather | FAILED (as it must): - the rain moves (0 pixels changed sharply) |
| og.Fog never extracted | `crates/og-runtime/src/atmosphere.rs` | og sky | FAILED (as it must): - sky.png matches sky.png: 35.828 % outliers (≤ 0.5 %), RMSE 0.15182 (≤ 0.01) |
| og.Water heights never handed to the renderer | `crates/og-runtime/src/atmosphere.rs` | og water | FAILED (as it must): - water.png matches water.png: 2.313 % outliers (≤ 0.5 %), RMSE 0.03257 (≤ 0.01) |
| fluids not drawn | `crates/og-runtime/src/atmosphere.rs` | og fluid | FAILED (as it must): - the basin holds blue fluid (rgb 75, 75, 85) |
| ragdoll drawn with the animation's pose | `crates/og-runtime/src/atmosphere.rs` | og joints | FAILED (as it must): - joints.png matches joints.png: 1.920 % outliers (≤ 0.5 %), RMSE 0.03657 (≤ 0.01) |
| setShaderParam does not reach the material | `crates/og-runtime/src/atmosphere.rs` | og shaders | FAILED (as it must): - shaders.png matches shaders.png: 23.776 % outliers (≤ 0.5 %), RMSE 0.12403 (≤ 0.01) |
| skills name a reference/ page the generated wiki does not have (the check removed) | `packages/knowledge/scripts/validate.ts` | validate.test.ts | FAILED (as it must) |

## Breaks the first version of a test missed

Each of these passed the first time it was applied. For nine of them the test was then strengthened and the same break run again; the table above shows the final result. The other two, the stale-handle joint and ropes rebuilt every 600 ticks, turned out to change no behaviour, so a real fault replaced each of them; the table explains both.

- spring stiffness dropped
- a joint is kept when its body was rebuilt (stale handle)
- a joint to a non-body goes to the world anchor
- character slope limit ignored
- 2D car never drives
- rope start does not follow its entity
- fluid has no pressure
- rope state outside the snapshot (rebuilt each tick)
- ropes outside the snapshot (not saved)
- day-night light does not fall at night
- surface_color hook never called
