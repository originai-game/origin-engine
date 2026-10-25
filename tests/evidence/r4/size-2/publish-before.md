# publish, before (shipping `wasm` profile; scripts/wasm-size-report.sh -n 60, then twiggy 0.8.0 top per crate)

```
publish.wasm: 19363271 B raw, 13157392 B after wasm-opt, 3561039 B brotli -q 11 (84.9 % of wasm_core_brotli_bytes, 4194304 B)
```

Per crate: code bodies and the data section, 13,063,038 B raw; "brotli alone" is each group compressed by itself (they sum to 3,763,532 B, more than the module's brotli)

| Crate | Functions | Raw bytes | Raw share | Brotli alone | Brotli share |
|---|---:|---:|---:|---:|---:|
| og_runtime | 652 | 1,488,584 | 11.40 % | 443,313 | 11.78 % |
| [data] | 1 | 1,518,870 | 11.63 % | 372,857 | 9.91 % |
| og_host_web (exports) | 360 | 1,221,620 | 9.35 % | 342,867 | 9.11 % |
| og_render | 678 | 1,077,440 | 8.25 % | 317,554 | 8.44 % |
| rapier3d | 341 | 825,130 | 6.32 % | 268,363 | 7.13 % |
| og_physics | 271 | 826,410 | 6.33 % | 237,969 | 6.32 % |
| parry3d | 985 | 770,952 | 5.90 % | 205,596 | 5.46 % |
| naga | 1,323 | 658,882 | 5.04 % | 192,846 | 5.12 % |
| wgpu | 359 | 426,882 | 3.27 % | 144,961 | 3.85 % |
| og_ui | 234 | 336,741 | 2.58 % | 110,526 | 2.94 % |
| std | 1,106 | 423,252 | 3.24 % | 96,999 | 2.58 % |
| wgpu_core | 753 | 371,797 | 2.85 % | 96,064 | 2.55 % |
| og_anim | 243 | 284,500 | 2.18 % | 85,879 | 2.28 % |
| wgpu_hal | 311 | 191,087 | 1.46 % | 65,627 | 1.74 % |
| og_ecs | 418 | 224,251 | 1.72 % | 59,445 | 1.58 % |
| og_nav | 85 | 150,744 | 1.15 % | 50,494 | 1.34 % |
| bincode | 89 | 168,984 | 1.29 % | 46,246 | 1.23 % |
| og_cook | 269 | 151,582 | 1.16 % | 43,877 | 1.17 % |
| ttf_parser | 296 | 129,803 | 0.99 % | 42,807 | 1.14 % |
| rustybuzz | 302 | 108,685 | 0.83 % | 40,420 | 1.07 % |
| serde_json | 199 | 195,237 | 1.49 % | 39,229 | 1.04 % |
| [unnamed] | 375 | 140,815 | 1.08 % | 38,489 | 1.02 % |
| og_audio | 82 | 94,683 | 0.72 % | 34,443 | 0.92 % |
| og_probe | 50 | 99,277 | 0.76 % | 34,364 | 0.91 % |
| og_core | 359 | 157,685 | 1.21 % | 33,230 | 0.88 % |
| og_ai | 65 | 80,811 | 0.62 % | 27,450 | 0.73 % |
| js_sys | 58 | 78,677 | 0.60 % | 26,519 | 0.70 % |
| og_geom | 67 | 86,181 | 0.66 % | 26,252 | 0.70 % |
| og_host_web | 76 | 102,072 | 0.78 % | 25,500 | 0.68 % |
| og_tilemap | 29 | 46,971 | 0.36 % | 16,184 | 0.43 % |
| og_timeline | 54 | 71,951 | 0.55 % | 16,049 | 0.43 % |
| og_net | 39 | 40,178 | 0.31 % | 13,403 | 0.36 % |
| og_input | 36 | 48,394 | 0.37 % | 13,172 | 0.35 % |
| og_i18n | 69 | 32,719 | 0.25 % | 12,790 | 0.34 % |
| og_tween | 37 | 34,768 | 0.27 % | 11,707 | 0.31 % |
| miniz_oxide | 31 | 25,965 | 0.20 % | 11,370 | 0.30 % |
| spade | 22 | 39,376 | 0.30 % | 10,963 | 0.29 % |
| indexmap | 108 | 48,116 | 0.37 % | 10,393 | 0.28 % |
| glam | 45 | 29,285 | 0.22 % | 9,822 | 0.26 % |
| wgpu_types | 240 | 28,308 | 0.22 % | 9,174 | 0.24 % |
| codespan_reporting | 58 | 21,441 | 0.16 % | 8,522 | 0.23 % |
| og_lanes | 67 | 32,175 | 0.25 % | 7,870 | 0.21 % |
| compiler_builtins | 18 | 18,778 | 0.14 % | 7,435 | 0.20 % |
| [other] | 74 | 17,882 | 0.14 % | 6,705 | 0.18 % |
| robust | 4 | 14,548 | 0.11 % | 4,264 | 0.11 % |
| nalgebra | 13 | 8,921 | 0.07 % | 3,908 | 0.10 % |
| libm | 11 | 9,166 | 0.07 % | 3,310 | 0.09 % |
| naga_types | 49 | 7,931 | 0.06 % | 3,281 | 0.09 % |
| og_blob | 28 | 13,076 | 0.10 % | 3,127 | 0.08 % |
| glow | 78 | 11,493 | 0.09 % | 3,062 | 0.08 % |
| smallvec | 47 | 9,890 | 0.08 % | 2,686 | 0.07 % |
| dlmalloc | 6 | 7,397 | 0.06 % | 2,473 | 0.07 % |
| arrayvec | 77 | 4,955 | 0.04 % | 2,197 | 0.06 % |
| unicode_bidi | 7 | 4,753 | 0.04 % | 2,047 | 0.05 % |
| serde_core | 25 | 6,303 | 0.05 % | 2,044 | 0.05 % |
| rustc_hash | 24 | 4,202 | 0.03 % | 1,565 | 0.04 % |
| wasm_bindgen | 44 | 4,070 | 0.03 % | 1,288 | 0.03 % |
| parking_lot | 6 | 3,871 | 0.03 % | 1,166 | 0.03 % |
| glamx | 4 | 2,191 | 0.02 % | 1,121 | 0.03 % |
| zmij | 1 | 1,753 | 0.01 % | 979 | 0.03 % |
| parking_lot_core | 8 | 1,823 | 0.01 % | 945 | 0.03 % |
| half | 21 | 1,842 | 0.01 % | 862 | 0.02 % |
| unicode_segmentation | 5 | 1,367 | 0.01 % | 701 | 0.02 % |
| foldhash | 5 | 3,233 | 0.02 % | 700 | 0.02 % |
| bit_set | 6 | 1,203 | 0.01 % | 696 | 0.02 % |
| web_sys | 99 | 2,147 | 0.02 % | 632 | 0.02 % |
| __rustc | 5 | 1,353 | 0.01 % | 601 | 0.02 % |
| ena | 4 | 1,631 | 0.01 % | 599 | 0.02 % |
| once_cell | 4 | 834 | 0.01 % | 486 | 0.01 % |
| wgpu_naga_bridge | 2 | 731 | 0.01 % | 428 | 0.01 % |
| raw_window_handle | 18 | 1,148 | 0.01 % | 403 | 0.01 % |
| bit_vec | 6 | 613 | 0.00 % | 382 | 0.01 % |
| itoa | 1 | 554 | 0.00 % | 283 | 0.01 % |
| lock_api | 2 | 389 | 0.00 % | 277 | 0.01 % |
| unicode_width | 1 | 336 | 0.00 % | 263 | 0.01 % |
| slotmap | 2 | 355 | 0.00 % | 250 | 0.01 % |
| bitflags | 4 | 402 | 0.00 % | 190 | 0.01 % |
| bytemuck | 5 | 198 | 0.00 % | 185 | 0.00 % |
| log | 2 | 188 | 0.00 % | 166 | 0.00 % |
| unicode_properties | 2 | 171 | 0.00 % | 157 | 0.00 % |
| serde_arrays | 1 | 59 | 0.00 % | 63 | 0.00 % |

Top 60 functions by raw bytes

| Raw bytes | Brotli alone | Crate | Function |
|---:|---:|---|---|
| 455,635 | 157,023 | og_render | `<og_render::Renderer>::render` |
| 360,413 | 124,046 | og_physics | `og_physics::schedule::{closure#0}` |
| 252,723 | 66,354 | og_host_web (exports) | `og_engine_create multivalue shim` |
| 250,943 | 80,872 | og_runtime | `og_runtime::extract::extract` |
| 127,875 | 44,805 | wgpu | `<wgpu::backend::wgpu_core::CoreCommandEncoder as wgpu::dispatch::CommandEncoderInterface>::finish` |
| 123,297 | 34,940 | bincode | `bincode::internal::deserialize_seed::<core::marker::PhantomData<og_physics::dim3::sim::Sim>, bincode::config::WithOtherTrailing<bincode::config::DefaultOptio...` |
| 99,392 | 33,244 | rapier3d | `rapier3d::dynamics::solver::staged_island_solver::worker::run_worker` |
| 89,409 | 30,039 | og_runtime | `og_runtime::modules::mount_back::{closure#0}` |
| 73,747 | 28,247 | rapier3d | `<rapier3d::pipeline::physics_pipeline::PhysicsPipeline>::detect_collisions` |
| 72,905 | 14,733 | og_physics | `<og_physics::PhysicsState as og_ecs::state::StateBlock>::save` |
| 72,488 | 24,393 | js_sys | `js_sys::futures::future_to_promise::<og_host_web::canvas::_::__wasm_bindgen_generated_og_web_attach_canvas::{closure#0}>::{closure#0}::{closure#0}` |
| 55,297 | 21,933 | og_ui | `<og_ui::text::FontMetrics>::run_one` |
| 55,174 | 18,265 | og_host_web (exports) | `og_render_csg_geometry multivalue shim` |
| 49,958 | 18,480 | wgpu_hal | `<wgpu_hal::gles::Device>::create_pipeline` |
| 48,500 | 9,006 | og_physics | `og_physics::gather::gather` |
| 47,680 | 16,794 | og_runtime | `og_runtime::timeline::decode` |
| 46,456 | 15,364 | og_host_web (exports) | `og_room_deliver` |
| 43,349 | 6,342 | og_host_web | `core::ptr::drop_glue::<og_host_web::canvas::CanvasBackend>` |
| 38,141 | 11,186 | og_anim | `og_anim::rig2d::parse_body` |
| 37,231 | 7,636 | naga | `<naga::back::glsl::writer::Writer<&mut alloc::string::String>>::write_expr` |
| 36,137 | 12,739 | rapier3d | `rapier3d::dynamics::solver::staged_island_solver::solve::solve_pass` |
| 31,425 | 12,132 | naga | `<naga::valid::Validator>::validate_impl` |
| 31,284 | 12,530 | og_runtime | `og_runtime::assets::spawn_scene` |
| 30,638 | 12,109 | og_runtime | `og_runtime::timeline::install::{closure#5}` |
| 30,091 | 7,022 | og_input | `<og_input::sink::InputLayer as og_input::sink::InputSink>::attach` |
| 30,084 | 10,888 | og_runtime | `og_runtime::anim::load_animation` |
| 29,655 | 11,242 | wgpu | `<wgpu::backend::wgpu_core::CoreQueue as wgpu::dispatch::QueueInterface>::submit` |
| 28,599 | 11,169 | og_runtime | `<og_runtime::describe::RuntimeView as og_probe::hooks::SceneView>::pick` |
| 28,068 | 8,949 | naga | `<naga::front::wgsl::lower::Lowerer>::call` |
| 27,907 | 9,687 | naga | `<naga::valid::Validator>::validate_function` |
| 27,620 | 10,575 | og_runtime | `og_runtime::stream::fetch` |
| 27,392 | 10,472 | og_audio | `og_audio::system::run` |
| 27,245 | 10,566 | og_probe | `<og_probe::probe::Probe>::execute` |
| 26,209 | 10,113 | naga | `<naga::front::wgsl::Frontend>::parse` |
| 25,707 | 8,931 | og_runtime | `og_runtime::engine::answer_call` |
| 25,479 | 10,232 | wgpu | `<wgpu::backend::wgpu_core::CoreDevice as wgpu::dispatch::DeviceInterface>::create_render_pipeline` |
| 24,885 | 7,866 | og_runtime | `og_runtime::modules::load_style` |
| 24,457 | 3,889 | rapier3d | `core::slice::sort::unstable::quicksort::quicksort::<(&rapier3d::dynamics::island_manager::persistent::JointLinkKey, &(u32, u32)), <[(&rapier3d::dynamics::isl...` |
| 23,736 | 5,234 | naga | `<naga::front::wgsl::error::Error>::as_parse_error` |
| 23,723 | 9,008 | og_render | `<og_render::terrain::TerrainRenderer>::sync` |
| 22,731 | 8,631 | og_anim | `<og_anim::rig2d::Rig2dAsset>::pose_of` |
| 22,702 | 8,900 | og_runtime | `og_runtime::assets::add_gltf` |
| 22,699 | 8,965 | ttf_parser | `<ttf_parser::Face>::parse` |
| 22,561 | 8,527 | og_anim | `og_anim::system::install::{closure#0}` |
| 22,473 | 8,357 | og_host_web (exports) | `og_load_tilemap multivalue shim` |
| 22,455 | 9,586 | og_nav | `og_nav::system::run` |
| 22,229 | 8,567 | og_host_web (exports) | `og_anim_retarget multivalue shim` |
| 21,209 | 7,516 | rapier3d | `rapier3d::dynamics::solver::staged_island_solver::solve::solve_pass::{closure#2}` |
| 20,481 | 8,234 | og_host_web (exports) | `og_load_flipbook multivalue shim` |
| 20,445 | 7,358 | parry3d | `parry3d::query::sweep_toi::sweep_toi::sweep_time_of_impact` |
| 20,264 | 7,122 | og_host_web (exports) | `og_anim_graph multivalue shim` |
| 20,120 | 7,928 | og_ui | `<og_ui::ui::Ui>::edit` |
| 20,083 | 7,396 | og_nav | `<og_nav::mesh::NavMesh>::build_tile` |
| 19,805 | 4,970 | naga | `<naga::back::glsl::writer::Writer<&mut alloc::string::String>>::write_stmt` |
| 19,801 | 8,107 | og_physics | `og_physics::dim3::sim::shape` |
| 19,680 | 6,224 | naga | `<naga::valid::Validator>::validate_block_impl` |
| 19,575 | 6,496 | parry3d | `<parry3d::query::default_query_dispatcher::DefaultQueryDispatcher as parry3d::query::query_dispatcher::PersistentQueryDispatcher>::contact_manifold_convex_co...` |
| 19,269 | 6,373 | og_runtime | `og_runtime::platform::mount::{closure#1}` |
| 19,263 | 8,043 | og_ai | `og_ai::system::install::{closure#0}` |
| 19,139 | 4,396 | spade | `spade::delaunay_core::math::contained_in_circumference::<f64>` |

twiggy top, per crate (shallow bytes; names and producers left out): 13,157,115 B

| Crate | Items | Shallow bytes | Share |
|---|---:|---:|---:|
| [data] | 3489 | 1,518,868 | 11.54 % |
| og_runtime | 652 | 1,490,442 | 11.33 % |
| og_host_web (exports) | 360 | 1,222,701 | 9.29 % |
| og_render | 678 | 1,079,336 | 8.20 % |
| og_physics | 271 | 827,156 | 6.29 % |
| rapier3d | 341 | 826,166 | 6.28 % |
| parry3d | 985 | 773,605 | 5.88 % |
| naga | 1323 | 662,048 | 5.03 % |
| wgpu | 359 | 427,791 | 3.25 % |
| std | 1106 | 425,949 | 3.24 % |
| wgpu_core | 753 | 373,719 | 2.84 % |
| og_ui | 234 | 337,422 | 2.56 % |
| og_anim | 243 | 285,177 | 2.17 % |
| og_ecs | 418 | 225,461 | 1.71 % |
| serde_json | 199 | 195,789 | 1.49 % |
| wgpu_hal | 311 | 191,843 | 1.46 % |
| bincode | 89 | 169,239 | 1.29 % |
| og_core | 359 | 158,589 | 1.21 % |
| og_cook | 269 | 152,248 | 1.16 % |
| og_nav | 85 | 150,986 | 1.15 % |
| [unnamed] | 375 | 141,856 | 1.08 % |
| ttf_parser | 296 | 130,565 | 0.99 % |
| rustybuzz | 302 | 109,423 | 0.83 % |
| og_host_web | 76 | 102,285 | 0.78 % |
| og_probe | 50 | 99,427 | 0.76 % |
| og_audio | 82 | 94,905 | 0.72 % |
| og_geom | 67 | 86,379 | 0.66 % |
| og_ai | 65 | 80,988 | 0.62 % |
| js_sys | 58 | 78,808 | 0.60 % |
| og_timeline | 54 | 72,110 | 0.55 % |
| [wasm structure] | 1442 | 62,673 | 0.48 % |
| og_input | 36 | 48,485 | 0.37 % |
| indexmap | 108 | 48,385 | 0.37 % |
| og_tilemap | 29 | 47,051 | 0.36 % |
| og_net | 39 | 40,293 | 0.31 % |
| spade | 22 | 39,445 | 0.30 % |
| og_tween | 37 | 34,871 | 0.27 % |
| og_i18n | 69 | 32,899 | 0.25 % |
| og_lanes | 67 | 32,369 | 0.25 % |
| glam | 45 | 29,408 | 0.22 % |
| wgpu_types | 240 | 28,828 | 0.22 % |
| miniz_oxide | 31 | 26,040 | 0.20 % |
| codespan_reporting | 58 | 21,587 | 0.16 % |
| compiler_builtins | 18 | 18,834 | 0.14 % |
| [other] | 79 | 18,104 | 0.14 % |
| robust | 4 | 14,563 | 0.11 % |
| og_blob | 28 | 13,148 | 0.10 % |
| glow | 78 | 11,684 | 0.09 % |
| smallvec | 47 | 10,012 | 0.08 % |
| libm | 11 | 9,197 | 0.07 % |
| nalgebra | 13 | 8,958 | 0.07 % |
| naga_types | 49 | 8,036 | 0.06 % |
| dlmalloc | 6 | 7,415 | 0.06 % |
| serde_core | 25 | 6,365 | 0.05 % |
| arrayvec | 77 | 5,113 | 0.04 % |
| unicode_bidi | 7 | 4,771 | 0.04 % |
| rustc_hash | 24 | 4,256 | 0.03 % |
| wasm_bindgen | 44 | 4,169 | 0.03 % |
| parking_lot | 6 | 3,889 | 0.03 % |
| foldhash | 5 | 3,249 | 0.02 % |
| web_sys | 99 | 2,345 | 0.02 % |
| glamx | 4 | 2,203 | 0.02 % |
| half | 21 | 1,891 | 0.01 % |
| parking_lot_core | 8 | 1,842 | 0.01 % |
| zmij | 1 | 1,757 | 0.01 % |
| ena | 4 | 1,642 | 0.01 % |
| unicode_segmentation | 5 | 1,380 | 0.01 % |
| __rustc | 5 | 1,365 | 0.01 % |
| bit_set | 6 | 1,219 | 0.01 % |
| raw_window_handle | 18 | 1,185 | 0.01 % |
| once_cell | 4 | 843 | 0.01 % |
| wgpu_naga_bridge | 2 | 738 | 0.01 % |
| bit_vec | 6 | 626 | 0.00 % |
| itoa | 1 | 558 | 0.00 % |
| bitflags | 4 | 412 | 0.00 % |
| lock_api | 2 | 394 | 0.00 % |
| slotmap | 2 | 360 | 0.00 % |
| unicode_width | 1 | 339 | 0.00 % |
| bytemuck | 5 | 208 | 0.00 % |
| log | 2 | 193 | 0.00 % |
| unicode_properties | 2 | 176 | 0.00 % |
| serde_arrays | 1 | 61 | 0.00 % |
