# publish2d, before (shipping `wasm` profile; scripts/wasm-size-report.sh -n 60, then twiggy 0.8.0 top per crate)

```
publish2d.wasm: 18197125 B raw, 12159793 B after wasm-opt, 3291397 B brotli -q 11 (78.5 % of wasm_core_brotli_bytes, 4194304 B)
```

Per crate: code bodies and the data section, 12,066,853 B raw; "brotli alone" is each group compressed by itself (they sum to 3,488,738 B, more than the module's brotli)

| Crate | Functions | Raw bytes | Raw share | Brotli alone | Brotli share |
|---|---:|---:|---:|---:|---:|
| og_runtime | 621 | 1,412,167 | 11.70 % | 421,810 | 12.09 % |
| [data] | 1 | 1,453,585 | 12.05 % | 362,140 | 10.38 % |
| og_host_web (exports) | 360 | 1,198,257 | 9.93 % | 335,488 | 9.62 % |
| og_render | 597 | 897,998 | 7.44 % | 264,489 | 7.58 % |
| og_physics | 261 | 796,795 | 6.60 % | 229,324 | 6.57 % |
| naga | 1,323 | 658,882 | 5.46 % | 192,874 | 5.53 % |
| rapier2d | 309 | 561,448 | 4.65 % | 183,179 | 5.25 % |
| wgpu | 359 | 426,883 | 3.54 % | 144,870 | 4.15 % |
| parry2d | 782 | 455,739 | 3.78 % | 128,385 | 3.68 % |
| og_ui | 234 | 336,836 | 2.79 % | 110,391 | 3.16 % |
| wgpu_core | 753 | 371,797 | 3.08 % | 96,338 | 2.76 % |
| std | 1,077 | 403,422 | 3.34 % | 92,028 | 2.64 % |
| og_anim | 244 | 284,510 | 2.36 % | 86,049 | 2.47 % |
| wgpu_hal | 310 | 190,097 | 1.58 % | 65,250 | 1.87 % |
| og_ecs | 418 | 224,215 | 1.86 % | 59,483 | 1.71 % |
| og_nav | 84 | 150,731 | 1.25 % | 50,340 | 1.44 % |
| bincode | 82 | 173,737 | 1.44 % | 48,540 | 1.39 % |
| ttf_parser | 296 | 129,803 | 1.08 % | 42,804 | 1.23 % |
| rustybuzz | 302 | 108,687 | 0.90 % | 40,315 | 1.16 % |
| serde_json | 199 | 195,237 | 1.62 % | 39,175 | 1.12 % |
| og_cook | 256 | 135,657 | 1.12 % | 39,127 | 1.12 % |
| [unnamed] | 372 | 140,264 | 1.16 % | 38,609 | 1.11 % |
| og_audio | 82 | 94,681 | 0.78 % | 34,517 | 0.99 % |
| og_probe | 50 | 99,275 | 0.82 % | 34,410 | 0.99 % |
| og_core | 356 | 157,651 | 1.31 % | 33,199 | 0.95 % |
| og_ai | 65 | 80,809 | 0.67 % | 27,440 | 0.79 % |
| js_sys | 57 | 79,694 | 0.66 % | 26,835 | 0.77 % |
| og_geom | 67 | 86,181 | 0.71 % | 26,176 | 0.75 % |
| og_host_web | 76 | 101,580 | 0.84 % | 25,382 | 0.73 % |
| og_tilemap | 29 | 46,972 | 0.39 % | 16,277 | 0.47 % |
| og_timeline | 54 | 71,951 | 0.60 % | 16,047 | 0.46 % |
| og_net | 39 | 40,270 | 0.33 % | 13,399 | 0.38 % |
| og_input | 36 | 48,394 | 0.40 % | 13,164 | 0.38 % |
| og_i18n | 69 | 32,719 | 0.27 % | 12,777 | 0.37 % |
| og_tween | 37 | 34,767 | 0.29 % | 11,718 | 0.34 % |
| miniz_oxide | 31 | 25,965 | 0.22 % | 11,366 | 0.33 % |
| spade | 23 | 39,531 | 0.33 % | 11,001 | 0.32 % |
| indexmap | 105 | 46,512 | 0.39 % | 10,430 | 0.30 % |
| wgpu_types | 240 | 28,308 | 0.23 % | 9,156 | 0.26 % |
| glam | 41 | 26,849 | 0.22 % | 9,057 | 0.26 % |
| codespan_reporting | 59 | 21,445 | 0.18 % | 8,530 | 0.24 % |
| og_lanes | 61 | 29,766 | 0.25 % | 7,426 | 0.21 % |
| compiler_builtins | 18 | 18,778 | 0.16 % | 7,411 | 0.21 % |
| [other] | 73 | 17,428 | 0.14 % | 6,668 | 0.19 % |
| robust | 4 | 14,548 | 0.12 % | 4,226 | 0.12 % |
| libm | 11 | 9,166 | 0.08 % | 3,305 | 0.09 % |
| naga_types | 49 | 7,931 | 0.07 % | 3,266 | 0.09 % |
| og_blob | 27 | 13,983 | 0.12 % | 3,177 | 0.09 % |
| glow | 78 | 11,493 | 0.10 % | 3,052 | 0.09 % |
| smallvec | 47 | 9,890 | 0.08 % | 2,699 | 0.08 % |
| arrayvec | 82 | 5,737 | 0.05 % | 2,536 | 0.07 % |
| dlmalloc | 6 | 7,397 | 0.06 % | 2,499 | 0.07 % |
| nalgebra | 8 | 5,431 | 0.05 % | 2,400 | 0.07 % |
| unicode_bidi | 7 | 4,753 | 0.04 % | 2,048 | 0.06 % |
| serde_core | 25 | 6,303 | 0.05 % | 2,044 | 0.06 % |
| rustc_hash | 24 | 4,202 | 0.03 % | 1,561 | 0.04 % |
| wasm_bindgen | 44 | 4,070 | 0.03 % | 1,287 | 0.04 % |
| parking_lot | 6 | 3,871 | 0.03 % | 1,163 | 0.03 % |
| zmij | 1 | 1,753 | 0.01 % | 982 | 0.03 % |
| parking_lot_core | 8 | 1,823 | 0.02 % | 940 | 0.03 % |
| half | 21 | 1,842 | 0.02 % | 862 | 0.02 % |
| unicode_segmentation | 5 | 1,367 | 0.01 % | 701 | 0.02 % |
| foldhash | 5 | 3,233 | 0.03 % | 696 | 0.02 % |
| bit_set | 6 | 1,203 | 0.01 % | 691 | 0.02 % |
| web_sys | 99 | 2,147 | 0.02 % | 616 | 0.02 % |
| __rustc | 5 | 1,353 | 0.01 % | 596 | 0.02 % |
| glamx | 3 | 1,100 | 0.01 % | 515 | 0.01 % |
| once_cell | 4 | 834 | 0.01 % | 486 | 0.01 % |
| wgpu_naga_bridge | 2 | 731 | 0.01 % | 427 | 0.01 % |
| raw_window_handle | 18 | 1,148 | 0.01 % | 403 | 0.01 % |
| bit_vec | 6 | 613 | 0.01 % | 380 | 0.01 % |
| itoa | 1 | 554 | 0.00 % | 283 | 0.01 % |
| lock_api | 2 | 389 | 0.00 % | 279 | 0.01 % |
| unicode_width | 1 | 336 | 0.00 % | 261 | 0.01 % |
| slotmap | 2 | 355 | 0.00 % | 249 | 0.01 % |
| bitflags | 4 | 402 | 0.00 % | 193 | 0.01 % |
| bytemuck | 4 | 242 | 0.00 % | 177 | 0.01 % |
| log | 2 | 188 | 0.00 % | 167 | 0.00 % |
| unicode_properties | 2 | 171 | 0.00 % | 152 | 0.00 % |
| serde_arrays | 1 | 21 | 0.00 % | 25 | 0.00 % |

Top 60 functions by raw bytes

| Raw bytes | Brotli alone | Crate | Function |
|---:|---:|---|---|
| 408,336 | 138,387 | og_render | `<og_render::Renderer>::render` |
| 362,144 | 122,658 | og_physics | `og_physics::dim2::sim::step_here` |
| 252,092 | 65,869 | og_host_web (exports) | `og_engine_create multivalue shim` |
| 216,267 | 69,422 | og_runtime | `og_runtime::extract::extract` |
| 127,876 | 44,857 | wgpu | `<wgpu::backend::wgpu_core::CoreCommandEncoder as wgpu::dispatch::CommandEncoderInterface>::finish` |
| 116,700 | 33,346 | bincode | `bincode::internal::deserialize_seed::<core::marker::PhantomData<og_physics::dim2::sim::Sim>, bincode::config::WithOtherTrailing<bincode::config::DefaultOptio...` |
| 89,409 | 30,073 | og_runtime | `og_runtime::modules::mount_back::{closure#0}` |
| 73,529 | 24,648 | js_sys | `js_sys::futures::future_to_promise::<og_host_web::canvas::_::__wasm_bindgen_generated_og_web_attach_canvas::{closure#0}>::{closure#0}::{closure#0}` |
| 71,878 | 14,113 | og_physics | `<og_physics::PhysicsState as og_ecs::state::StateBlock>::save` |
| 64,586 | 24,653 | rapier2d | `<rapier2d::pipeline::physics_pipeline::PhysicsPipeline>::detect_collisions` |
| 55,297 | 21,947 | og_ui | `<og_ui::text::FontMetrics>::run_one` |
| 55,174 | 18,298 | og_host_web (exports) | `og_render_csg_geometry multivalue shim` |
| 49,958 | 18,463 | wgpu_hal | `<wgpu_hal::gles::Device>::create_pipeline` |
| 48,500 | 9,025 | og_physics | `og_physics::gather::gather` |
| 47,692 | 16,797 | og_runtime | `og_runtime::timeline::decode` |
| 46,456 | 15,347 | og_host_web (exports) | `og_room_deliver` |
| 43,349 | 6,301 | og_host_web | `core::ptr::drop_glue::<og_host_web::canvas::CanvasBackend>` |
| 38,137 | 11,191 | og_anim | `og_anim::rig2d::parse_body` |
| 37,231 | 7,586 | naga | `<naga::back::glsl::writer::Writer<&mut alloc::string::String>>::write_expr` |
| 31,425 | 12,151 | naga | `<naga::valid::Validator>::validate_impl` |
| 31,284 | 12,542 | og_runtime | `og_runtime::assets::spawn_scene` |
| 30,639 | 12,086 | og_runtime | `og_runtime::timeline::install::{closure#5}` |
| 30,091 | 7,037 | og_input | `<og_input::sink::InputLayer as og_input::sink::InputSink>::attach` |
| 30,084 | 10,868 | og_runtime | `og_runtime::anim::load_animation` |
| 29,655 | 11,223 | wgpu | `<wgpu::backend::wgpu_core::CoreQueue as wgpu::dispatch::QueueInterface>::submit` |
| 29,364 | 10,294 | rapier2d | `rapier2d::dynamics::solver::staged_island_solver::solve::solve_pass` |
| 28,599 | 11,159 | og_runtime | `<og_runtime::describe::RuntimeView as og_probe::hooks::SceneView>::pick` |
| 28,068 | 8,970 | naga | `<naga::front::wgsl::lower::Lowerer>::call` |
| 27,907 | 9,687 | naga | `<naga::valid::Validator>::validate_function` |
| 27,786 | 10,607 | og_runtime | `og_runtime::stream::fetch` |
| 27,390 | 10,462 | og_audio | `og_audio::system::run` |
| 27,243 | 10,576 | og_probe | `<og_probe::probe::Probe>::execute` |
| 26,209 | 10,128 | naga | `<naga::front::wgsl::Frontend>::parse` |
| 25,705 | 8,949 | og_runtime | `og_runtime::engine::answer_call` |
| 25,479 | 10,227 | wgpu | `<wgpu::backend::wgpu_core::CoreDevice as wgpu::dispatch::DeviceInterface>::create_render_pipeline` |
| 24,885 | 7,875 | og_runtime | `og_runtime::modules::load_style` |
| 24,457 | 3,866 | rapier2d | `core::slice::sort::unstable::quicksort::quicksort::<(&rapier2d::dynamics::island_manager::persistent::JointLinkKey, &(u32, u32)), <[(&rapier2d::dynamics::isl...` |
| 23,736 | 5,208 | naga | `<naga::front::wgsl::error::Error>::as_parse_error` |
| 22,731 | 8,640 | og_anim | `<og_anim::rig2d::Rig2dAsset>::pose_of` |
| 22,702 | 8,876 | og_runtime | `og_runtime::assets::add_gltf` |
| 22,699 | 8,972 | ttf_parser | `<ttf_parser::Face>::parse` |
| 22,561 | 8,528 | og_anim | `og_anim::system::install::{closure#0}` |
| 22,473 | 8,361 | og_host_web (exports) | `og_load_tilemap multivalue shim` |
| 22,453 | 9,559 | og_nav | `og_nav::system::run` |
| 22,229 | 8,578 | og_host_web (exports) | `og_anim_retarget multivalue shim` |
| 20,481 | 8,231 | og_host_web (exports) | `og_load_flipbook multivalue shim` |
| 20,264 | 7,111 | og_host_web (exports) | `og_anim_graph multivalue shim` |
| 20,120 | 7,881 | og_ui | `<og_ui::ui::Ui>::edit` |
| 20,083 | 7,394 | og_nav | `<og_nav::mesh::NavMesh>::build_tile` |
| 19,805 | 4,965 | naga | `<naga::back::glsl::writer::Writer<&mut alloc::string::String>>::write_stmt` |
| 19,680 | 6,218 | naga | `<naga::valid::Validator>::validate_block_impl` |
| 19,269 | 6,356 | og_runtime | `og_runtime::platform::mount::{closure#1}` |
| 19,263 | 8,051 | og_ai | `og_ai::system::install::{closure#0}` |
| 19,139 | 4,316 | spade | `spade::delaunay_core::math::contained_in_circumference::<f64>` |
| 19,042 | 6,171 | og_cook | `<og_cook::runtime::Bundle>::model` |
| 18,993 | 6,740 | wgpu_hal | `<wgpu_hal::gles::Queue as wgpu_hal::dynamic::queue::DynQueue>::submit` |
| 18,986 | 7,826 | wgpu | `<wgpu::backend::wgpu_core::CoreAdapter as wgpu::dispatch::AdapterInterface>::request_device` |
| 18,631 | 7,242 | wgpu_hal | `<wgpu_hal::gles::web::Instance as wgpu_hal::dynamic::instance::DynInstance>::enumerate_adapters` |
| 18,624 | 7,640 | og_host_web (exports) | `og_engine_frame multivalue shim` |
| 18,602 | 7,656 | og_ui | `og_ui::text::bidi_runs` |

twiggy top, per crate (shallow bytes; names and producers left out): 12,159,516 B

| Crate | Items | Shallow bytes | Share |
|---|---:|---:|---:|
| [data] | 3466 | 1,453,583 | 11.95 % |
| og_runtime | 621 | 1,413,934 | 11.63 % |
| og_host_web (exports) | 360 | 1,199,344 | 9.86 % |
| og_render | 597 | 899,662 | 7.40 % |
| og_physics | 261 | 797,515 | 6.56 % |
| naga | 1323 | 662,048 | 5.44 % |
| rapier2d | 309 | 562,386 | 4.63 % |
| parry2d | 782 | 457,793 | 3.76 % |
| wgpu | 359 | 427,792 | 3.52 % |
| std | 1077 | 406,041 | 3.34 % |
| wgpu_core | 753 | 373,719 | 3.07 % |
| og_ui | 234 | 337,518 | 2.78 % |
| og_anim | 244 | 285,190 | 2.35 % |
| og_ecs | 418 | 225,425 | 1.85 % |
| serde_json | 199 | 195,789 | 1.61 % |
| wgpu_hal | 310 | 190,853 | 1.57 % |
| bincode | 82 | 173,974 | 1.43 % |
| og_core | 356 | 158,549 | 1.30 % |
| og_nav | 84 | 150,971 | 1.24 % |
| [unnamed] | 372 | 141,308 | 1.16 % |
| og_cook | 256 | 136,282 | 1.12 % |
| ttf_parser | 296 | 130,565 | 1.07 % |
| rustybuzz | 302 | 109,425 | 0.90 % |
| og_host_web | 76 | 101,793 | 0.84 % |
| og_probe | 50 | 99,425 | 0.82 % |
| og_audio | 82 | 94,903 | 0.78 % |
| og_geom | 67 | 86,379 | 0.71 % |
| og_ai | 65 | 80,986 | 0.67 % |
| js_sys | 57 | 79,823 | 0.66 % |
| og_timeline | 54 | 72,110 | 0.59 % |
| [wasm structure] | 1453 | 62,497 | 0.51 % |
| og_input | 36 | 48,485 | 0.40 % |
| og_tilemap | 29 | 47,052 | 0.39 % |
| indexmap | 105 | 46,772 | 0.38 % |
| og_net | 39 | 40,385 | 0.33 % |
| spade | 23 | 39,603 | 0.33 % |
| og_tween | 37 | 34,870 | 0.29 % |
| og_i18n | 69 | 32,899 | 0.27 % |
| og_lanes | 61 | 29,942 | 0.25 % |
| wgpu_types | 240 | 28,828 | 0.24 % |
| glam | 41 | 26,958 | 0.22 % |
| miniz_oxide | 31 | 26,040 | 0.21 % |
| codespan_reporting | 59 | 21,593 | 0.18 % |
| compiler_builtins | 18 | 18,834 | 0.15 % |
| [other] | 78 | 17,647 | 0.15 % |
| robust | 4 | 14,563 | 0.12 % |
| og_blob | 27 | 14,054 | 0.12 % |
| glow | 78 | 11,684 | 0.10 % |
| smallvec | 47 | 10,012 | 0.08 % |
| libm | 11 | 9,197 | 0.08 % |
| naga_types | 49 | 8,036 | 0.07 % |
| dlmalloc | 6 | 7,415 | 0.06 % |
| serde_core | 25 | 6,365 | 0.05 % |
| arrayvec | 82 | 5,907 | 0.05 % |
| nalgebra | 8 | 5,454 | 0.04 % |
| unicode_bidi | 7 | 4,771 | 0.04 % |
| rustc_hash | 24 | 4,256 | 0.04 % |
| wasm_bindgen | 44 | 4,170 | 0.03 % |
| parking_lot | 6 | 3,889 | 0.03 % |
| foldhash | 5 | 3,249 | 0.03 % |
| web_sys | 99 | 2,345 | 0.02 % |
| half | 21 | 1,891 | 0.02 % |
| parking_lot_core | 8 | 1,842 | 0.02 % |
| zmij | 1 | 1,757 | 0.01 % |
| unicode_segmentation | 5 | 1,380 | 0.01 % |
| __rustc | 5 | 1,365 | 0.01 % |
| bit_set | 6 | 1,219 | 0.01 % |
| raw_window_handle | 18 | 1,185 | 0.01 % |
| glamx | 3 | 1,108 | 0.01 % |
| once_cell | 4 | 843 | 0.01 % |
| wgpu_naga_bridge | 2 | 738 | 0.01 % |
| bit_vec | 6 | 626 | 0.01 % |
| itoa | 1 | 558 | 0.00 % |
| bitflags | 4 | 412 | 0.00 % |
| lock_api | 2 | 394 | 0.00 % |
| slotmap | 2 | 360 | 0.00 % |
| unicode_width | 1 | 339 | 0.00 % |
| bytemuck | 4 | 250 | 0.00 % |
| log | 2 | 193 | 0.00 % |
| unicode_properties | 2 | 176 | 0.00 % |
| serde_arrays | 1 | 23 | 0.00 % |
