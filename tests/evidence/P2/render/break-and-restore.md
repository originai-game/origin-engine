# Break and restore, phase 2 render round

Each new test was run against a deliberately broken build of the code it
covers, failed, and passed again once the code was restored.

| Test | The break | How it failed |
|---|---|---|
| `lights.rs` `moving_the_depth_slices_changes_nothing` | `clusters.rs` `light_box`: a light only in the depth slice of its centre (`z0 = z1 = slice(d)`) | 123,287 pixels changed with the near and far planes moved |
| `lights.rs` `tile_boundaries_leave_no_step` | the screen rectangle one tile narrower on each side in x | a step of 66.7 at column 80 against 0.58 beside it |
| `lights.rs` `past_the_cluster_cap_the_brightest_lights_stay` | clusters filled in scene order instead of by importance | the capped frame differs from the 64 brightest by 19 |
| `lights.rs` `past_max_lights_the_faint_far_ones_are_dropped` | the first 256 visible lights kept instead of the most important | max difference 240 |
| `lights.rs` `lights_outside_the_view_are_culled` | the light frustum test widened by 1e9 m (nothing culled) | stats (74, 0) instead of (64, 10) |
| `moon.rs` `the_phase_lights_the_disc` | the moon lit from `(-sin θ, 0, -cos θ)` | the first quarter lit the left half |
| `moon.rs` `the_moon_sets_below_the_horizon` | the disc and halo drawn without the horizon factors | 40 pixels of the set moon showed |
| `moon.rs` `the_size_sets_the_disc` | the disc's coordinates divided by a fixed `sin(0.04)` instead of its radius | area ratio 1.00 |
| `og-render/tests/tier.rs` `software_rasterizers_are_known_by_name_too` | `is_software_adapter` by device type alone | llvmpipe reported as `Other` was not `vulkan-cpu` |
| `materials.rs` `a_non_uniform_scale_bends_normals_correctly` | normals through the model matrix (the old `normalize(model * n)`) | 21,644 pixels apart, RMSE 0.043 |
| `materials.rs` `normal_maps_follow_the_vertex_tangent_frame` | the frame from screen derivatives instead of the vertex tangents | coarse 1,663 steps against 1,257 on the fine sphere |
| `materials.rs` `mirrored_uvs_keep_the_map_the_right_way_round` | the same derivative frame (it was inverted in a y-down framebuffer) | plain 57, mirrored 222 |
| `materials.rs` `generate_tangents_follows_the_uvs`, `a_normal_maps_green_points_up_the_image` | `generate_tangents` with the bitangent along +v | the quad's w wrong; green up 41 against green down 222 |
| `materials.rs` `clearcoat_adds_a_glossy_layer` | the coat ignored in the shader | no highlight added |
| `hooks.rs` `an_offset_mesh_…`, `an_offset_skinned_mesh_…` | the shadow pass's `vs_hooked` without the material shader's offset | 7,066 and 18,613 pixels apart (the shadows where the mesh was) |
| the same two | hooked batches drawn by the plain prepass (no offset in the depth, AO and outline) | 3,937 and 12,926 pixels apart |
| `hooks.rs` `an_offset_skinned_mesh_…`, `skinned_hooks_match_their_reference` | skinned batches without their material shader (the old behaviour) | 70,545 pixels apart; the reference off by RMSE 0.087 |
| fx-showcase `tests/skinned.og.ts` | `og.render.setMaterialShader` answering ok without giving the material its shader | no gold bands; the frame off its reference |
