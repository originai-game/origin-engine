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
