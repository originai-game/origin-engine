# og-render M6 test run (8d160fe content, at 49ed935)

`CARGO_TARGET_DIR=target/agent-qa cargo test --release -p og-tests -- --nocapture` on Apple M4 Pro, Metal, macOS, 2026-10-18 (load average 15-25 from other agents' builds).

| Test binary | Result |
|---|---|
| `tests/gfx.rs` | ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out, 2.93s |
| `tests/graph.rs` | ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out, 0.08s |
| `tests/perf.rs` | ok. 0 passed; 0 failed; 4 ignored; 0 measured; 0 filtered out, 0.00s |
| `tests/render.rs` | ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out, 0.95s |
| `tests/wasm.rs` | ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out, 46.45s |

Tests in `tests/render.rs` (M6 render): the_default_material_is_the_schema_default, the_material_schema_matches_the_idl, malformed_texture_uploads_are_refused, the_mobile_preset_is_reported, transparent_draws_are_sorted_back_to_front, transparent_instances_of_one_material_are_sorted_too, capture_is_bounded_and_cancellable, a_reflected_material_edit_reaches_the_frame, gltf_materials_draw_as_their_standard_form, unlit_materials_ignore_lighting, double_sided_materials_draw_back_faces, shadow_map_size_sets_the_shadow_resolution, cascade_count_is_one_to_three, compressed_formats_are_chosen_per_tier_and_draw_like_rgba8.

Tests in `tests/graph.rs` (the render graph object): a_missing_target_is_refused_with_the_pass_and_the_name, a_cycle_is_refused_with_its_passes, transients_alias_when_their_lifetimes_do_not_overlap, passes_run_in_data_flow_order, the_showcase_frame_is_recorded_from_its_graph.

## Every gfx scene against its committed Metal reference

`cargo run --release -p og-tests --bin og-gfx -- --all --frames 120` (frame times are under load; the M3 baselines are in tests/evidence/M3/perf-baselines.json):

```
gltf-reference: 1280x720 tier=metal adapter="Apple M4 Pro" aa=Off draws=113 batches=38 instances=926 culled=5 triangles=878827 shadow_passes=3 shadow_casters=969 lights=0 sprites=0 glyphs=0 sprite_draws=0 setup+first=1313.1ms frame=1.966ms cpu=0.698ms (120 frames) vs reference: rmse=0.00000 max_diff=0 outliers=0
spheres-20k: 1280x720 tier=metal adapter="Apple M4 Pro" aa=Off draws=7 batches=2 instances=20001 culled=972 triangles=7680002 shadow_passes=3 shadow_casters=16719 lights=0 sprites=0 glyphs=0 sprite_draws=0 setup+first=68.1ms frame=7.558ms cpu=1.349ms (120 frames) vs reference: rmse=0.00000 max_diff=0 outliers=0
cascades-200m: 1280x720 tier=metal adapter="Apple M4 Pro" aa=Off draws=20 batches=5 instances=24 culled=0 triangles=278 shadow_passes=3 shadow_casters=50 lights=0 sprites=0 glyphs=0 sprite_draws=0 setup+first=50.8ms frame=0.906ms cpu=0.396ms (120 frames) vs reference: rmse=0.00000 max_diff=0 outliers=0
point-spot-lights: 1280x720 tier=metal adapter="Apple M4 Pro" aa=Off draws=23 batches=3 instances=9 culled=0 triangles=4670 shadow_passes=7 shadow_casters=32 lights=7 sprites=0 glyphs=0 sprite_draws=0 setup+first=47.4ms frame=1.071ms cpu=0.648ms (120 frames) vs reference: rmse=0.00000 max_diff=0 outliers=0
taa: 1280x720 tier=metal adapter="Apple M4 Pro" aa=Taa draws=19 batches=4 instances=19 culled=0 triangles=1742 shadow_passes=3 shadow_casters=46 lights=0 sprites=0 glyphs=0 sprite_draws=0 setup+first=153.4ms frame=0.771ms cpu=0.412ms (120 frames) vs reference: rmse=0.00000 max_diff=0 outliers=0
msaa: 1280x720 tier=metal adapter="Apple M4 Pro" aa=Taa draws=19 batches=4 instances=19 culled=0 triangles=1742 shadow_passes=3 shadow_casters=46 lights=0 sprites=0 glyphs=0 sprite_draws=0 setup+first=52.0ms frame=0.858ms cpu=0.466ms (120 frames) (no reference on this tier)
bloom: 1280x720 tier=metal adapter="Apple M4 Pro" aa=Off draws=31 batches=6 instances=6 culled=0 triangles=7682 shadow_passes=3 shadow_casters=13 lights=0 sprites=0 glyphs=0 sprite_draws=0 setup+first=50.6ms frame=3.256ms cpu=1.260ms (120 frames) vs reference: rmse=0.00000 max_diff=0 outliers=0
exposure: 1280x720 tier=metal adapter="Apple M4 Pro" aa=Off draws=14 batches=3 instances=11 culled=0 triangles=7742 shadow_passes=3 shadow_casters=23 lights=0 sprites=0 glyphs=0 sprite_draws=0 setup+first=47.7ms frame=1.466ms cpu=0.700ms (120 frames) vs reference: rmse=0.00000 max_diff=0 outliers=0
sprites-20k: 1280x720 tier=metal adapter="Apple M4 Pro" aa=Off draws=4 batches=0 instances=0 culled=0 triangles=0 shadow_passes=0 shadow_casters=0 lights=0 sprites=20000 glyphs=0 sprite_draws=3 setup+first=47.1ms frame=0.653ms cpu=0.558ms (120 frames) vs reference: rmse=0.00000 max_diff=0 outliers=0
text-msdf: 1280x720 tier=metal adapter="Apple M4 Pro" aa=Off draws=2 batches=0 instances=0 culled=0 triangles=0 shadow_passes=0 shadow_casters=0 lights=0 sprites=0 glyphs=131 sprite_draws=1 setup+first=47.4ms frame=0.397ms cpu=0.164ms (120 frames) vs reference: rmse=0.00000 max_diff=0 outliers=0
sky-only: 1280x720 tier=metal adapter="Apple M4 Pro" aa=Off draws=2 batches=0 instances=0 culled=0 triangles=0 shadow_passes=0 shadow_casters=0 lights=0 sprites=0 glyphs=0 sprite_draws=0 setup+first=48.5ms frame=0.429ms cpu=0.153ms (120 frames) vs reference: rmse=0.00000 max_diff=0 outliers=0
showcase: 1280x720 tier=metal adapter="Apple M4 Pro" aa=Taa draws=252 batches=41 instances=79 culled=0 triangles=93557 shadow_passes=10 shadow_casters=235 lights=3 sprites=40 glyphs=16 sprite_draws=3 setup+first=119.5ms frame=5.322ms cpu=2.249ms (120 frames) vs reference: rmse=0.00000 max_diff=0 outliers=0
```

Every scene with a Metal reference is RMSE 0.00000 / 0 outliers: the graph rewrite and the material system left every M3 frame byte-identical. `msaa` has no reference on this tier (no-compute only). The `vulkan-cpu` references are as committed in 1e0dee6 and were not re-run here (that tier needs the E2B lavapipe sandbox); 8d160fe's message reports them passing.

## Frames read

- `render-showcase-metal.png` (og-gfx showcase --out): three tricksters lit orange, blue and green by local lights with bloom halos, the two dropships, the checker ground with cascaded and local shadows, the MSDF world text "Origin Engine 原点引擎" glowing in the sky, sprite rings and dots; byte-identical to references/metal/showcase.png and to tests/evidence/M3/showcase-metal.png (sha256 74b159885d0a66f7).
- `render-showcase-vulkan-cpu.png`: the committed lavapipe reference, the same picture (sha256 40f886d3cdce3101, unchanged since M3).
- `render-shadow-map-256.png` / `render-shadow-map-2048.png` (written by `shadow_map_size_sets_the_shadow_resolution`): a grey pole on a light plane casting a blue-grey shadow to the right; at 256² the shadow is soft-edged, narrower, shorter and detached from the pole's foot by a small gap; at 2048² it is crisp and starts at the foot. The test asserts the edge ramp is more than 3× wider at 256².

Not covered by a frame: transparency sorting, double-sided and unlit materials are asserted on pixels in `tests/render.rs` but no frame of them is saved; perf (`--ignored perf`) is reported in render-perf.md.
