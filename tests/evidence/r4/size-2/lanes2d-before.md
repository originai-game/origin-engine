# lanes2d, before (shipping `wasm` profile; scripts/wasm-size-report.sh -n 60, then twiggy 0.8.0 top per crate)

```
lanes2d.wasm: 18784249 B raw, 12484766 B after wasm-opt, 3327994 B brotli -q 11 (79.3 % of wasm_core_brotli_bytes, 4194304 B)
```

Per crate: code bodies and the data section, 12,383,917 B raw; "brotli alone" is each group compressed by itself (they sum to 3,532,311 B, more than the module's brotli)

| Crate | Functions | Raw bytes | Raw share | Brotli alone | Brotli share |
|---|---:|---:|---:|---:|---:|
| og_runtime | 647 | 1,396,828 | 11.28 % | 416,752 | 11.80 % |
| [data] | 1 | 1,531,078 | 12.36 % | 353,005 | 9.99 % |
| og_host_web (exports) | 363 | 1,224,274 | 9.89 % | 334,242 | 9.46 % |
| rapier2d | 343 | 806,607 | 6.51 % | 265,410 | 7.51 % |
| og_render | 598 | 894,595 | 7.22 % | 264,581 | 7.49 % |
| naga | 1,325 | 649,585 | 5.25 % | 192,780 | 5.46 % |
| wgpu | 360 | 413,604 | 3.34 % | 142,695 | 4.04 % |
| og_physics | 261 | 534,136 | 4.31 % | 141,013 | 3.99 % |
| parry2d | 797 | 473,590 | 3.82 % | 131,687 | 3.73 % |
| og_ui | 237 | 336,966 | 2.72 % | 110,554 | 3.13 % |
| std | 1,115 | 414,887 | 3.35 % | 95,578 | 2.71 % |
| wgpu_core | 759 | 363,654 | 2.94 % | 95,298 | 2.70 % |
| og_anim | 246 | 282,213 | 2.28 % | 85,579 | 2.42 % |
| wgpu_hal | 309 | 187,824 | 1.52 % | 65,039 | 1.84 % |
| og_ecs | 418 | 223,175 | 1.80 % | 59,401 | 1.68 % |
| og_nav | 83 | 150,498 | 1.22 % | 50,293 | 1.42 % |
| bincode | 82 | 173,810 | 1.40 % | 48,504 | 1.37 % |
| ttf_parser | 294 | 129,835 | 1.05 % | 42,780 | 1.21 % |
| [unnamed] | 394 | 153,756 | 1.24 % | 41,461 | 1.17 % |
| rustybuzz | 295 | 109,071 | 0.88 % | 40,555 | 1.15 % |
| serde_json | 201 | 196,195 | 1.58 % | 39,265 | 1.11 % |
| og_cook | 257 | 118,846 | 0.96 % | 36,904 | 1.04 % |
| og_audio | 85 | 96,556 | 0.78 % | 34,666 | 0.98 % |
| og_probe | 50 | 97,074 | 0.78 % | 33,860 | 0.96 % |
| og_core | 355 | 157,577 | 1.27 % | 33,213 | 0.94 % |
| og_ai | 64 | 79,619 | 0.64 % | 27,318 | 0.77 % |
| js_sys | 69 | 80,762 | 0.65 % | 27,277 | 0.77 % |
| og_geom | 67 | 86,217 | 0.70 % | 26,215 | 0.74 % |
| og_host_web | 77 | 99,637 | 0.80 % | 25,468 | 0.72 % |
| rayon | 36 | 80,900 | 0.65 % | 25,361 | 0.72 % |
| rayon_core | 271 | 138,811 | 1.12 % | 20,970 | 0.59 % |
| og_tilemap | 28 | 46,802 | 0.38 % | 16,095 | 0.46 % |
| og_timeline | 54 | 71,930 | 0.58 % | 16,082 | 0.46 % |
| og_lanes | 82 | 48,131 | 0.39 % | 14,778 | 0.42 % |
| og_net | 41 | 39,759 | 0.32 % | 13,232 | 0.37 % |
| og_input | 36 | 48,384 | 0.39 % | 13,162 | 0.37 % |
| og_i18n | 69 | 32,841 | 0.27 % | 12,779 | 0.36 % |
| og_tween | 37 | 34,604 | 0.28 % | 11,658 | 0.33 % |
| miniz_oxide | 31 | 25,948 | 0.21 % | 11,342 | 0.32 % |
| spade | 23 | 39,530 | 0.32 % | 11,056 | 0.31 % |
| indexmap | 109 | 46,539 | 0.38 % | 10,271 | 0.29 % |
| [other] | 83 | 24,422 | 0.20 % | 9,365 | 0.27 % |
| wgpu_types | 240 | 28,502 | 0.23 % | 9,216 | 0.26 % |
| glam | 41 | 26,982 | 0.22 % | 9,128 | 0.26 % |
| codespan_reporting | 58 | 21,325 | 0.17 % | 8,447 | 0.24 % |
| compiler_builtins | 18 | 17,568 | 0.14 % | 6,991 | 0.20 % |
| robust | 4 | 14,548 | 0.12 % | 4,288 | 0.12 % |
| libm | 11 | 9,166 | 0.07 % | 3,324 | 0.09 % |
| naga_types | 49 | 7,933 | 0.06 % | 3,297 | 0.09 % |
| og_blob | 26 | 14,324 | 0.12 % | 3,156 | 0.09 % |
| glow | 76 | 11,429 | 0.09 % | 3,031 | 0.09 % |
| smallvec | 47 | 9,850 | 0.08 % | 2,678 | 0.08 % |
| arrayvec | 82 | 5,728 | 0.05 % | 2,555 | 0.07 % |
| nalgebra | 8 | 5,433 | 0.04 % | 2,395 | 0.07 % |
| dlmalloc | 7 | 6,557 | 0.05 % | 2,294 | 0.06 % |
| serde_core | 25 | 6,450 | 0.05 % | 2,099 | 0.06 % |
| unicode_bidi | 7 | 4,753 | 0.04 % | 2,048 | 0.06 % |
| crossbeam_channel | 17 | 7,487 | 0.06 % | 1,999 | 0.06 % |
| crossbeam_epoch | 17 | 4,257 | 0.03 % | 1,708 | 0.05 % |
| rustc_hash | 24 | 4,202 | 0.03 % | 1,566 | 0.04 % |
| wasm_bindgen | 53 | 4,449 | 0.04 % | 1,402 | 0.04 % |
| parking_lot | 6 | 4,040 | 0.03 % | 1,288 | 0.04 % |
| crossbeam_deque | 8 | 4,809 | 0.04 % | 1,065 | 0.03 % |
| zmij | 1 | 1,753 | 0.01 % | 980 | 0.03 % |
| parking_lot_core | 10 | 1,815 | 0.01 % | 973 | 0.03 % |
| half | 21 | 1,842 | 0.01 % | 865 | 0.02 % |
| unicode_segmentation | 5 | 1,367 | 0.01 % | 695 | 0.02 % |
| foldhash | 5 | 3,224 | 0.03 % | 693 | 0.02 % |
| bit_set | 6 | 1,203 | 0.01 % | 693 | 0.02 % |
| __rustc | 5 | 1,492 | 0.01 % | 648 | 0.02 % |
| web_sys | 99 | 2,137 | 0.02 % | 641 | 0.02 % |
| glamx | 3 | 1,100 | 0.01 % | 512 | 0.01 % |
| wasm_bindgen_rayon | 2 | 1,121 | 0.01 % | 492 | 0.01 % |
| once_cell | 4 | 769 | 0.01 % | 462 | 0.01 % |
| wgpu_naga_bridge | 2 | 731 | 0.01 % | 426 | 0.01 % |
| raw_window_handle | 18 | 1,148 | 0.01 % | 412 | 0.01 % |
| bit_vec | 6 | 613 | 0.00 % | 385 | 0.01 % |
| itoa | 1 | 554 | 0.00 % | 278 | 0.01 % |
| unicode_width | 1 | 336 | 0.00 % | 261 | 0.01 % |
| lock_api | 2 | 346 | 0.00 % | 256 | 0.01 % |
| slotmap | 2 | 356 | 0.00 % | 248 | 0.01 % |
| bitflags | 4 | 402 | 0.00 % | 194 | 0.01 % |
| bytemuck | 5 | 197 | 0.00 % | 187 | 0.01 % |
| log | 2 | 193 | 0.00 % | 154 | 0.00 % |
| unicode_properties | 2 | 171 | 0.00 % | 150 | 0.00 % |
| wasm_sync | 1 | 85 | 0.00 % | 85 | 0.00 % |
| crossbeam_utils | 1 | 79 | 0.00 % | 77 | 0.00 % |
| serde_arrays | 1 | 21 | 0.00 % | 25 | 0.00 % |

Top 60 functions by raw bytes

| Raw bytes | Brotli alone | Crate | Function |
|---:|---:|---|---|
| 406,507 | 138,298 | og_render | `<og_render::Renderer>::render` |
| 248,003 | 64,878 | og_host_web (exports) | `og_engine_create multivalue shim` |
| 215,382 | 69,930 | og_runtime | `og_runtime::extract::extract` |
| 159,367 | 58,454 | rapier2d | `<rapier2d::pipeline::physics_pipeline::PhysicsPipeline>::step_inner` |
| 123,987 | 44,528 | wgpu | `<wgpu::backend::wgpu_core::CoreCommandEncoder as wgpu::dispatch::CommandEncoderInterface>::finish` |
| 116,821 | 33,503 | bincode | `bincode::internal::deserialize_seed::<core::marker::PhantomData<og_physics::dim2::sim::Sim>, bincode::config::WithOtherTrailing<bincode::config::DefaultOptio...` |
| 101,899 | 34,987 | og_physics | `og_physics::dim2::sim::step_here` |
| 89,244 | 30,076 | og_runtime | `og_runtime::modules::mount_back::{closure#0}` |
| 72,735 | 24,379 | js_sys | `js_sys::futures::future_to_promise::<og_host_web::canvas::_::__wasm_bindgen_generated_og_web_attach_canvas::{closure#0}>::{closure#0}::{closure#0}` |
| 71,858 | 14,045 | og_physics | `<og_physics::PhysicsState as og_ecs::state::StateBlock>::save` |
| 61,861 | 20,007 | rapier2d | `rapier2d::dynamics::solver::staged_island_solver::worker::run_worker` |
| 55,572 | 22,037 | og_ui | `<og_ui::text::FontMetrics>::run_one` |
| 55,566 | 15,257 | og_host_web (exports) | `og_room_deliver` |
| 55,493 | 18,034 | og_host_web (exports) | `og_render_csg_geometry multivalue shim` |
| 49,248 | 18,466 | wgpu_hal | `<wgpu_hal::gles::Device>::create_pipeline` |
| 49,222 | 9,206 | og_physics | `og_physics::gather::gather` |
| 47,318 | 16,568 | og_runtime | `og_runtime::timeline::decode` |
| 41,809 | 6,256 | og_host_web | `core::ptr::drop_glue::<og_host_web::canvas::CanvasBackend>` |
| 40,021 | 15,137 | rapier2d | `<rapier2d::pipeline::physics_pipeline::PhysicsPipeline>::detect_collisions` |
| 37,073 | 10,999 | og_anim | `og_anim::rig2d::parse_body` |
| 31,430 | 12,162 | naga | `<naga::valid::Validator>::validate_impl` |
| 31,121 | 12,512 | og_runtime | `og_runtime::assets::spawn_scene` |
| 30,466 | 11,976 | og_runtime | `og_runtime::timeline::install::{closure#5}` |
| 30,091 | 7,012 | og_input | `<og_input::sink::InputLayer as og_input::sink::InputSink>::attach` |
| 30,061 | 10,960 | og_runtime | `og_runtime::anim::load_animation` |
| 29,596 | 10,354 | rapier2d | `rapier2d::dynamics::solver::staged_island_solver::solve::solve_pass` |
| 29,498 | 7,177 | naga | `<naga::back::glsl::writer::Writer<&mut alloc::string::String>>::write_expr` |
| 28,692 | 11,071 | wgpu | `<wgpu::backend::wgpu_core::CoreQueue as wgpu::dispatch::QueueInterface>::submit` |
| 28,097 | 8,974 | naga | `<naga::front::wgsl::lower::Lowerer>::call` |
| 28,012 | 9,732 | naga | `<naga::valid::Validator>::validate_function` |
| 27,499 | 10,812 | og_runtime | `<og_runtime::describe::RuntimeView as og_probe::hooks::SceneView>::pick` |
| 27,258 | 10,520 | og_runtime | `og_runtime::stream::fetch` |
| 27,236 | 10,309 | og_audio | `og_audio::system::run` |
| 26,234 | 10,158 | naga | `<naga::front::wgsl::Frontend>::parse` |
| 25,465 | 10,040 | og_probe | `<og_probe::probe::Probe>::execute` |
| 25,094 | 8,669 | og_runtime | `og_runtime::engine::answer_call` |
| 25,068 | 10,118 | wgpu | `<wgpu::backend::wgpu_core::CoreDevice as wgpu::dispatch::DeviceInterface>::create_render_pipeline` |
| 24,457 | 3,866 | rapier2d | `core::slice::sort::unstable::quicksort::quicksort::<(&rapier2d::dynamics::island_manager::persistent::JointLinkKey, &(u32, u32)), <[(&rapier2d::dynamics::isl...` |
| 24,332 | 7,636 | og_runtime | `og_runtime::modules::load_style` |
| 23,984 | 5,278 | naga | `<naga::front::wgsl::error::Error>::as_parse_error` |
| 23,004 | 8,894 | og_runtime | `og_runtime::assets::add_gltf` |
| 22,735 | 8,631 | og_anim | `<og_anim::rig2d::Rig2dAsset>::pose_of` |
| 22,699 | 8,968 | ttf_parser | `<ttf_parser::Face>::parse` |
| 22,592 | 8,615 | og_anim | `og_anim::system::install::{closure#0}` |
| 22,538 | 8,384 | og_host_web (exports) | `og_load_tilemap multivalue shim` |
| 22,512 | 9,662 | og_nav | `og_nav::system::run` |
| 22,150 | 9,285 | rapier2d | `<rapier2d::geometry::narrow_phase::NarrowPhase>::compute_contacts::{closure#3}` |
| 22,059 | 8,516 | og_host_web (exports) | `og_anim_retarget multivalue shim` |
| 20,307 | 7,067 | og_host_web (exports) | `og_anim_graph multivalue shim` |
| 20,102 | 7,877 | og_ui | `<og_ui::ui::Ui>::edit` |
| 20,058 | 7,393 | og_nav | `<og_nav::mesh::NavMesh>::build_tile` |
| 19,894 | 8,060 | og_host_web (exports) | `og_load_flipbook multivalue shim` |
| 19,768 | 7,017 | og_physics | `og_physics::schedule::{closure#0}` |
| 19,700 | 6,227 | naga | `<naga::valid::Validator>::validate_block_impl` |
| 19,416 | 8,490 | rayon | `rayon::iter::plumbing::bridge_producer_consumer::helper::<rayon::iter::enumerate::EnumerateProducer<rayon::slice::IterMutProducer<rapier2d::dynamics::solver:...` |
| 19,254 | 6,220 | og_cook | `<og_cook::runtime::Bundle>::model` |
| 19,139 | 4,364 | spade | `spade::delaunay_core::math::contained_in_circumference::<f64>` |
| 19,125 | 8,153 | og_ai | `og_ai::system::install::{closure#0}` |
| 18,906 | 6,787 | wgpu_hal | `<wgpu_hal::gles::Queue as wgpu_hal::dynamic::queue::DynQueue>::submit` |
| 18,639 | 7,248 | wgpu_hal | `<wgpu_hal::gles::web::Instance as wgpu_hal::dynamic::instance::DynInstance>::enumerate_adapters` |

twiggy top, per crate (shallow bytes; names and producers left out): 12,484,496 B

| Crate | Items | Shallow bytes | Share |
|---|---:|---:|---:|
| [data] | 3 | 1,531,077 | 12.26 % |
| og_runtime | 647 | 1,398,664 | 11.20 % |
| og_host_web (exports) | 363 | 1,225,368 | 9.82 % |
| og_render | 598 | 896,254 | 7.18 % |
| rapier2d | 343 | 807,645 | 6.47 % |
| naga | 1325 | 652,757 | 5.23 % |
| og_physics | 261 | 534,856 | 4.28 % |
| parry2d | 797 | 475,689 | 3.81 % |
| std | 1115 | 417,609 | 3.35 % |
| wgpu | 360 | 414,512 | 3.32 % |
| wgpu_core | 759 | 365,585 | 2.93 % |
| og_ui | 237 | 337,655 | 2.70 % |
| og_anim | 246 | 282,898 | 2.27 % |
| og_ecs | 418 | 224,384 | 1.80 % |
| serde_json | 201 | 196,752 | 1.58 % |
| wgpu_hal | 309 | 188,578 | 1.51 % |
| bincode | 82 | 174,047 | 1.39 % |
| og_core | 355 | 158,473 | 1.27 % |
| [unnamed] | 394 | 154,866 | 1.24 % |
| og_nav | 83 | 150,735 | 1.21 % |
| rayon_core | 271 | 139,590 | 1.12 % |
| ttf_parser | 294 | 130,593 | 1.05 % |
| og_cook | 257 | 119,460 | 0.96 % |
| rustybuzz | 295 | 109,799 | 0.88 % |
| og_host_web | 77 | 99,855 | 0.80 % |
| og_probe | 50 | 97,224 | 0.78 % |
| og_audio | 85 | 96,784 | 0.78 % |
| og_geom | 67 | 86,415 | 0.69 % |
| rayon | 36 | 81,006 | 0.65 % |
| js_sys | 69 | 80,917 | 0.65 % |
| og_ai | 64 | 79,794 | 0.64 % |
| og_timeline | 54 | 72,089 | 0.58 % |
| [wasm structure] | 1499 | 68,864 | 0.55 % |
| og_input | 36 | 48,475 | 0.39 % |
| og_lanes | 82 | 48,366 | 0.39 % |
| og_tilemap | 28 | 46,880 | 0.38 % |
| indexmap | 109 | 46,807 | 0.37 % |
| og_net | 41 | 39,881 | 0.32 % |
| spade | 23 | 39,602 | 0.32 % |
| og_tween | 37 | 34,707 | 0.28 % |
| og_i18n | 69 | 33,022 | 0.26 % |
| wgpu_types | 240 | 29,022 | 0.23 % |
| glam | 41 | 27,091 | 0.22 % |
| miniz_oxide | 31 | 26,023 | 0.21 % |
| [other] | 88 | 24,667 | 0.20 % |
| codespan_reporting | 58 | 21,471 | 0.17 % |
| compiler_builtins | 18 | 17,625 | 0.14 % |
| robust | 4 | 14,563 | 0.12 % |
| og_blob | 26 | 14,393 | 0.12 % |
| glow | 76 | 11,616 | 0.09 % |
| smallvec | 47 | 9,972 | 0.08 % |
| libm | 11 | 9,197 | 0.07 % |
| naga_types | 49 | 8,038 | 0.06 % |
| crossbeam_channel | 17 | 7,533 | 0.06 % |
| dlmalloc | 7 | 6,576 | 0.05 % |
| serde_core | 25 | 6,513 | 0.05 % |
| arrayvec | 82 | 5,898 | 0.05 % |
| nalgebra | 8 | 5,456 | 0.04 % |
| crossbeam_deque | 8 | 4,833 | 0.04 % |
| unicode_bidi | 7 | 4,771 | 0.04 % |
| wasm_bindgen | 53 | 4,569 | 0.04 % |
| crossbeam_epoch | 17 | 4,300 | 0.03 % |
| rustc_hash | 24 | 4,256 | 0.03 % |
| parking_lot | 6 | 4,058 | 0.03 % |
| foldhash | 5 | 3,240 | 0.03 % |
| web_sys | 99 | 2,335 | 0.02 % |
| half | 21 | 1,891 | 0.02 % |
| parking_lot_core | 10 | 1,839 | 0.01 % |
| zmij | 1 | 1,757 | 0.01 % |
| __rustc | 5 | 1,506 | 0.01 % |
| unicode_segmentation | 5 | 1,380 | 0.01 % |
| bit_set | 6 | 1,219 | 0.01 % |
| raw_window_handle | 18 | 1,185 | 0.01 % |
| wasm_bindgen_rayon | 2 | 1,126 | 0.01 % |
| glamx | 3 | 1,108 | 0.01 % |
| once_cell | 4 | 778 | 0.01 % |
| wgpu_naga_bridge | 2 | 738 | 0.01 % |
| bit_vec | 6 | 626 | 0.01 % |
| itoa | 1 | 558 | 0.00 % |
| bitflags | 4 | 412 | 0.00 % |
| slotmap | 2 | 361 | 0.00 % |
| lock_api | 2 | 351 | 0.00 % |
| unicode_width | 1 | 339 | 0.00 % |
| bytemuck | 5 | 207 | 0.00 % |
| log | 2 | 198 | 0.00 % |
| unicode_properties | 2 | 176 | 0.00 % |
| wasm_sync | 1 | 87 | 0.00 % |
| crossbeam_utils | 1 | 81 | 0.00 % |
| serde_arrays | 1 | 23 | 0.00 % |
