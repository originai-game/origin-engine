# lanes2d, after (shipping `wasm` profile; scripts/wasm-size-report.sh -n 60, then twiggy 0.8.0 top per crate)

```
lanes2d.wasm: 18407265 B raw, 10952306 B after wasm-opt, 3072212 B brotli -q 11 (73.2 % of wasm_core_brotli_bytes, 4194304 B)
```

Per crate: code bodies and the data section, 10,847,884 B raw; "brotli alone" is each group compressed by itself (they sum to 3,228,898 B, more than the module's brotli)

| Crate | Functions | Raw bytes | Raw share | Brotli alone | Brotli share |
|---|---:|---:|---:|---:|---:|
| og_runtime | 957 | 1,064,324 | 9.81 % | 340,171 | 10.54 % |
| [data] | 1 | 1,397,035 | 12.88 % | 317,189 | 9.82 % |
| og_host_web (exports) | 382 | 1,056,524 | 9.74 % | 300,344 | 9.30 % |
| rapier2d | 364 | 827,211 | 7.63 % | 268,332 | 8.31 % |
| og_render | 611 | 726,235 | 6.69 % | 234,734 | 7.27 % |
| naga | 1,255 | 625,099 | 5.76 % | 185,831 | 5.76 % |
| parry2d | 799 | 461,815 | 4.26 % | 131,497 | 4.07 % |
| wgpu | 382 | 353,274 | 3.26 % | 127,374 | 3.94 % |
| og_physics | 270 | 479,645 | 4.42 % | 125,314 | 3.88 % |
| og_ui | 253 | 306,539 | 2.83 % | 105,075 | 3.25 % |
| std | 1,592 | 394,354 | 3.64 % | 103,681 | 3.21 % |
| wgpu_core | 951 | 329,076 | 3.03 % | 93,437 | 2.89 % |
| og_anim | 290 | 277,590 | 2.56 % | 82,805 | 2.56 % |
| wgpu_hal | 355 | 166,638 | 1.54 % | 61,509 | 1.90 % |
| og_nav | 88 | 148,259 | 1.37 % | 49,991 | 1.55 % |
| bincode | 72 | 174,364 | 1.61 % | 49,150 | 1.52 % |
| og_ecs | 461 | 179,230 | 1.65 % | 48,825 | 1.51 % |
| ttf_parser | 294 | 129,823 | 1.20 % | 42,877 | 1.33 % |
| rustybuzz | 307 | 109,333 | 1.01 % | 40,453 | 1.25 % |
| og_core | 516 | 157,875 | 1.46 % | 34,074 | 1.06 % |
| [unnamed] | 395 | 102,735 | 0.95 % | 30,879 | 0.96 % |
| og_cook | 281 | 96,460 | 0.89 % | 30,054 | 0.93 % |
| og_ai | 71 | 88,769 | 0.82 % | 29,355 | 0.91 % |
| og_audio | 103 | 71,059 | 0.66 % | 27,572 | 0.85 % |
| js_sys | 73 | 75,555 | 0.70 % | 27,502 | 0.85 % |
| serde_json | 252 | 80,096 | 0.74 % | 26,343 | 0.82 % |
| og_probe | 96 | 69,982 | 0.65 % | 26,184 | 0.81 % |
| rayon | 36 | 81,457 | 0.75 % | 25,756 | 0.80 % |
| rayon_core | 279 | 136,780 | 1.26 % | 21,258 | 0.66 % |
| og_host_web | 80 | 47,007 | 0.43 % | 17,386 | 0.54 % |
| og_geom | 60 | 42,945 | 0.40 % | 16,707 | 0.52 % |
| og_lanes | 83 | 46,997 | 0.43 % | 14,548 | 0.45 % |
| og_tilemap | 52 | 29,668 | 0.27 % | 12,231 | 0.38 % |
| og_input | 71 | 48,539 | 0.45 % | 12,020 | 0.37 % |
| glam | 76 | 30,829 | 0.28 % | 11,586 | 0.36 % |
| miniz_oxide | 31 | 26,078 | 0.24 % | 11,394 | 0.35 % |
| og_tween | 62 | 28,889 | 0.27 % | 10,828 | 0.34 % |
| og_i18n | 87 | 26,580 | 0.25 % | 10,602 | 0.33 % |
| og_net | 63 | 28,502 | 0.26 % | 10,415 | 0.32 % |
| wgpu_types | 283 | 29,299 | 0.27 % | 10,284 | 0.32 % |
| indexmap | 124 | 42,698 | 0.39 % | 9,896 | 0.31 % |
| codespan_reporting | 59 | 21,801 | 0.20 % | 8,760 | 0.27 % |
| [other] | 95 | 22,550 | 0.21 % | 8,384 | 0.26 % |
| spade | 44 | 23,661 | 0.22 % | 8,073 | 0.25 % |
| compiler_builtins | 37 | 17,989 | 0.17 % | 7,743 | 0.24 % |
| robust | 7 | 28,597 | 0.26 % | 7,656 | 0.24 % |
| og_timeline | 42 | 12,965 | 0.12 % | 5,637 | 0.17 % |
| libm | 13 | 8,992 | 0.08 % | 3,335 | 0.10 % |
| naga_types | 57 | 7,048 | 0.06 % | 3,291 | 0.10 % |
| og_blob | 33 | 9,867 | 0.09 % | 3,208 | 0.10 % |
| smallvec | 64 | 10,957 | 0.10 % | 3,042 | 0.09 % |
| arrayvec | 98 | 7,046 | 0.06 % | 2,963 | 0.09 % |
| glow | 77 | 9,691 | 0.09 % | 2,673 | 0.08 % |
| crossbeam_channel | 28 | 7,933 | 0.07 % | 2,483 | 0.08 % |
| nalgebra | 9 | 5,464 | 0.05 % | 2,395 | 0.07 % |
| dlmalloc | 22 | 5,813 | 0.05 % | 2,367 | 0.07 % |
| unicode_bidi | 13 | 5,831 | 0.05 % | 2,326 | 0.07 % |
| wasm_bindgen | 58 | 5,210 | 0.05 % | 1,850 | 0.06 % |
| crossbeam_epoch | 27 | 4,196 | 0.04 % | 1,518 | 0.05 % |
| rustc_hash | 27 | 3,545 | 0.03 % | 1,354 | 0.04 % |
| parking_lot | 6 | 3,801 | 0.04 % | 1,287 | 0.04 % |
| crossbeam_deque | 19 | 3,781 | 0.03 % | 1,234 | 0.04 % |
| serde_core | 24 | 3,326 | 0.03 % | 1,219 | 0.04 % |
| zmij | 1 | 2,148 | 0.02 % | 1,110 | 0.03 % |
| parking_lot_core | 11 | 1,729 | 0.02 % | 939 | 0.03 % |
| half | 21 | 1,831 | 0.02 % | 827 | 0.03 % |
| unicode_segmentation | 8 | 1,580 | 0.01 % | 812 | 0.03 % |
| foldhash | 4 | 3,041 | 0.03 % | 655 | 0.02 % |
| once_cell | 5 | 945 | 0.01 % | 542 | 0.02 % |
| glamx | 3 | 1,093 | 0.01 % | 526 | 0.02 % |
| wgpu_naga_bridge | 3 | 1,393 | 0.01 % | 520 | 0.02 % |
| wasm_bindgen_rayon | 3 | 1,084 | 0.01 % | 488 | 0.02 % |
| bitflags | 9 | 1,410 | 0.01 % | 478 | 0.01 % |
| bit_vec | 9 | 717 | 0.01 % | 440 | 0.01 % |
| web_sys | 27 | 980 | 0.01 % | 409 | 0.01 % |
| slotmap | 6 | 681 | 0.01 % | 408 | 0.01 % |
| raw_window_handle | 19 | 1,170 | 0.01 % | 403 | 0.01 % |
| bit_set | 6 | 590 | 0.01 % | 399 | 0.01 % |
| itoa | 3 | 457 | 0.00 % | 326 | 0.01 % |
| wasm_sync | 4 | 424 | 0.00 % | 251 | 0.01 % |
| __rustc | 5 | 316 | 0.00 % | 234 | 0.01 % |
| lock_api | 2 | 278 | 0.00 % | 231 | 0.01 % |
| log | 3 | 340 | 0.00 % | 203 | 0.01 % |
| bytemuck | 5 | 174 | 0.00 % | 172 | 0.01 % |
| unicode_properties | 2 | 171 | 0.00 % | 155 | 0.00 % |
| crossbeam_utils | 1 | 82 | 0.00 % | 86 | 0.00 % |
| serde_arrays | 1 | 19 | 0.00 % | 23 | 0.00 % |

Top 60 functions by raw bytes

| Raw bytes | Brotli alone | Crate | Function |
|---:|---:|---|---|
| 405,160 | 136,325 | og_render | `<og_render::Renderer>::render` |
| 238,572 | 60,479 | og_host_web (exports) | `og_engine_create multivalue shim` |
| 165,949 | 61,950 | og_runtime | `og_runtime::extract::extract` |
| 160,620 | 58,026 | rapier2d | `<rapier2d::pipeline::physics_pipeline::PhysicsPipeline>::step_inner` |
| 119,106 | 34,236 | bincode | `bincode::internal::deserialize_seed::<core::marker::PhantomData<og_physics::dim2::sim::Sim>, bincode::config::WithOtherTrailing<bincode::config::DefaultOptio...` |
| 104,495 | 35,256 | og_physics | `og_physics::dim2::sim::step_here` |
| 95,623 | 35,695 | wgpu | `<wgpu::backend::wgpu_core::CoreCommandEncoder as wgpu::dispatch::CommandEncoderInterface>::finish` |
| 84,870 | 28,147 | og_runtime | `og_runtime::modules::mount_back::{closure#0}` |
| 68,598 | 24,682 | js_sys | `js_sys::futures::future_to_promise::<og_host_web::canvas::_::__wasm_bindgen_generated_og_web_attach_canvas::{closure#0}>::{closure#0}::{closure#0}` |
| 61,568 | 19,936 | rapier2d | `rapier2d::dynamics::solver::staged_island_solver::worker::run_worker` |
| 55,984 | 22,377 | og_ui | `<og_ui::text::FontMetrics>::run_one` |
| 51,310 | 17,589 | og_host_web (exports) | `og_render_csg_geometry multivalue shim` |
| 49,841 | 9,251 | og_physics | `og_physics::gather::gather` |
| 41,752 | 16,724 | wgpu_hal | `<wgpu_hal::gles::Device>::create_pipeline` |
| 40,960 | 15,341 | rapier2d | `<rapier2d::pipeline::physics_pipeline::PhysicsPipeline>::detect_collisions` |
| 37,976 | 11,295 | og_anim | `og_anim::rig2d::parse_body` |
| 37,436 | 14,318 | og_runtime | `og_runtime::timeline::decode` |
| 34,246 | 12,407 | og_runtime | `og_runtime::engine::call_rpc` |
| 31,474 | 6,516 | og_physics | `<og_physics::PhysicsState as og_ecs::state::StateBlock>::save` |
| 30,888 | 11,846 | naga | `<naga::valid::Validator>::validate_impl` |
| 30,442 | 6,578 | og_input | `<og_input::sink::InputLayer as og_input::sink::InputSink>::attach` |
| 29,989 | 11,043 | og_runtime | `og_runtime::timeline::install::{closure#5}` |
| 29,442 | 10,181 | rapier2d | `rapier2d::dynamics::solver::staged_island_solver::solve::solve_pass` |
| 28,824 | 11,833 | og_runtime | `og_runtime::assets::spawn_scene` |
| 28,428 | 10,052 | og_host_web (exports) | `og_room_deliver` |
| 28,134 | 8,957 | naga | `<naga::front::wgsl::lower::Lowerer>::call` |
| 27,640 | 9,653 | naga | `<naga::valid::Validator>::validate_function` |
| 26,278 | 6,672 | naga | `<naga::back::glsl::writer::Writer<&mut alloc::string::String>>::write_expr` |
| 25,835 | 10,076 | naga | `<naga::front::wgsl::Frontend>::parse` |
| 24,464 | 3,868 | rapier2d | `core::slice::sort::unstable::quicksort::quicksort::<(&rapier2d::dynamics::island_manager::persistent::JointLinkKey, &(u32, u32)), <[(&rapier2d::dynamics::isl...` |
| 23,558 | 7,717 | og_runtime | `og_runtime::modules::load_style` |
| 23,500 | 5,169 | naga | `<naga::front::wgsl::error::Error>::as_parse_error` |
| 22,839 | 9,542 | og_nav | `og_nav::system::run` |
| 22,699 | 8,964 | ttf_parser | `<ttf_parser::Face>::parse` |
| 22,646 | 8,320 | og_anim | `<og_anim::rig2d::Rig2dAsset>::pose_of` |
| 22,499 | 8,500 | og_anim | `og_anim::system::install::{closure#0}` |
| 22,355 | 9,441 | wgpu | `<wgpu::backend::wgpu_core::CoreQueue as wgpu::dispatch::QueueInterface>::submit` |
| 21,948 | 9,316 | wgpu | `<wgpu::backend::wgpu_core::CoreDevice as wgpu::dispatch::DeviceInterface>::create_render_pipeline` |
| 21,853 | 8,768 | og_audio | `og_audio::system::install::{closure#0}` |
| 21,821 | 9,527 | og_runtime | `og_runtime::stream::fetch` |
| 21,794 | 9,100 | rapier2d | `<rapier2d::geometry::narrow_phase::NarrowPhase>::compute_contacts::{closure#3}` |
| 21,679 | 8,206 | og_render | `og_render::id_pass::render_ids_from` |
| 21,671 | 8,713 | og_probe | `<og_probe::probe::Probe>::execute` |
| 21,491 | 8,001 | og_runtime | `og_runtime::anim::load_animation` |
| 20,220 | 7,886 | og_ui | `<og_ui::ui::Ui>::edit` |
| 19,932 | 8,156 | og_ai | `og_ai::system::install::{closure#0}` |
| 19,691 | 6,182 | naga | `<naga::valid::Validator>::validate_block_impl` |
| 19,646 | 7,325 | og_nav | `<og_nav::mesh::NavMesh>::build_tile` |
| 19,515 | 6,959 | og_physics | `og_physics::schedule::{closure#0}` |
| 19,482 | 8,643 | rayon | `rayon::iter::plumbing::bridge_producer_consumer::helper::<rayon::iter::enumerate::EnumerateProducer<rayon::slice::IterMutProducer<rapier2d::dynamics::solver:...` |
| 19,299 | 7,773 | og_host_web (exports) | `og_load_flipbook multivalue shim` |
| 18,969 | 7,824 | og_ui | `og_ui::text::bidi_runs` |
| 18,861 | 6,070 | og_cook | `<og_cook::runtime::Bundle>::model` |
| 18,804 | 7,788 | og_host_web (exports) | `og_anim_retarget multivalue shim` |
| 18,300 | 7,507 | wgpu_hal | `<wgpu_hal::gles::web::Instance as wgpu_hal::dynamic::instance::DynInstance>::enumerate_adapters` |
| 17,828 | 6,374 | wgpu_hal | `<wgpu_hal::gles::Queue as wgpu_hal::dynamic::queue::DynQueue>::submit` |
| 17,601 | 6,075 | og_host_web (exports) | `og_load_tilemap multivalue shim` |
| 17,506 | 7,323 | rapier2d | `<rapier2d::dynamics::soft_body::soft_body_set::soft_body_set::SoftBodySet>::split_cluster` |
| 17,439 | 6,407 | og_host_web (exports) | `og_anim_graph multivalue shim` |
| 17,424 | 7,154 | og_runtime | `og_runtime::save::apply_keeping` |

twiggy top, per crate (shallow bytes; names and producers left out): 10,952,036 B

| Crate | Items | Shallow bytes | Share |
|---|---:|---:|---:|
| [data] | 3 | 1,397,034 | 12.76 % |
| og_runtime | 957 | 1,066,910 | 9.74 % |
| og_host_web (exports) | 382 | 1,057,676 | 9.66 % |
| rapier2d | 364 | 828,291 | 7.56 % |
| og_render | 611 | 727,847 | 6.65 % |
| naga | 1255 | 628,118 | 5.74 % |
| og_physics | 270 | 480,375 | 4.39 % |
| parry2d | 799 | 463,919 | 4.24 % |
| std | 1592 | 398,258 | 3.64 % |
| wgpu | 382 | 354,219 | 3.23 % |
| wgpu_core | 951 | 331,445 | 3.03 % |
| og_ui | 253 | 307,253 | 2.81 % |
| og_anim | 290 | 278,381 | 2.54 % |
| og_ecs | 461 | 180,526 | 1.65 % |
| bincode | 72 | 174,573 | 1.59 % |
| wgpu_hal | 355 | 167,475 | 1.53 % |
| og_core | 516 | 159,164 | 1.45 % |
| og_nav | 88 | 148,507 | 1.36 % |
| rayon_core | 279 | 137,575 | 1.26 % |
| ttf_parser | 294 | 130,581 | 1.19 % |
| rustybuzz | 307 | 110,082 | 1.01 % |
| [unnamed] | 395 | 103,774 | 0.95 % |
| og_cook | 281 | 97,104 | 0.89 % |
| og_ai | 71 | 88,965 | 0.81 % |
| rayon | 36 | 81,563 | 0.74 % |
| serde_json | 252 | 80,734 | 0.74 % |
| js_sys | 73 | 75,715 | 0.69 % |
| og_audio | 103 | 71,329 | 0.65 % |
| og_probe | 96 | 70,238 | 0.64 % |
| [wasm structure] | 1505 | 68,502 | 0.63 % |
| og_input | 71 | 48,697 | 0.44 % |
| og_lanes | 83 | 47,234 | 0.43 % |
| og_host_web | 80 | 47,216 | 0.43 % |
| og_geom | 60 | 43,124 | 0.39 % |
| indexmap | 124 | 42,995 | 0.39 % |
| glam | 76 | 31,029 | 0.28 % |
| wgpu_types | 283 | 29,917 | 0.27 % |
| og_tilemap | 52 | 29,800 | 0.27 % |
| og_tween | 62 | 29,053 | 0.27 % |
| og_net | 63 | 28,669 | 0.26 % |
| robust | 7 | 28,622 | 0.26 % |
| og_i18n | 87 | 26,800 | 0.24 % |
| miniz_oxide | 31 | 26,153 | 0.24 % |
| spade | 44 | 23,776 | 0.22 % |
| [other] | 100 | 22,815 | 0.21 % |
| codespan_reporting | 59 | 21,948 | 0.20 % |
| compiler_builtins | 37 | 18,103 | 0.17 % |
| og_timeline | 42 | 13,075 | 0.12 % |
| smallvec | 64 | 11,110 | 0.10 % |
| og_blob | 33 | 9,949 | 0.09 % |
| glow | 77 | 9,879 | 0.09 % |
| libm | 13 | 9,030 | 0.08 % |
| crossbeam_channel | 28 | 8,004 | 0.07 % |
| arrayvec | 98 | 7,254 | 0.07 % |
| naga_types | 57 | 7,174 | 0.07 % |
| dlmalloc | 22 | 5,866 | 0.05 % |
| unicode_bidi | 13 | 5,862 | 0.05 % |
| nalgebra | 9 | 5,489 | 0.05 % |
| wasm_bindgen | 58 | 5,341 | 0.05 % |
| crossbeam_epoch | 27 | 4,261 | 0.04 % |
| crossbeam_deque | 19 | 3,827 | 0.03 % |
| parking_lot | 6 | 3,819 | 0.03 % |
| rustc_hash | 27 | 3,605 | 0.03 % |
| serde_core | 24 | 3,382 | 0.03 % |
| foldhash | 4 | 3,053 | 0.03 % |
| zmij | 1 | 2,152 | 0.02 % |
| half | 21 | 1,880 | 0.02 % |
| parking_lot_core | 11 | 1,755 | 0.02 % |
| unicode_segmentation | 8 | 1,600 | 0.01 % |
| bitflags | 9 | 1,435 | 0.01 % |
| wgpu_naga_bridge | 3 | 1,401 | 0.01 % |
| raw_window_handle | 19 | 1,209 | 0.01 % |
| glamx | 3 | 1,101 | 0.01 % |
| wasm_bindgen_rayon | 3 | 1,091 | 0.01 % |
| web_sys | 27 | 1,034 | 0.01 % |
| once_cell | 5 | 958 | 0.01 % |
| bit_vec | 9 | 736 | 0.01 % |
| slotmap | 6 | 695 | 0.01 % |
| bit_set | 6 | 604 | 0.01 % |
| itoa | 3 | 465 | 0.00 % |
| wasm_sync | 4 | 433 | 0.00 % |
| log | 3 | 348 | 0.00 % |
| __rustc | 5 | 327 | 0.00 % |
| lock_api | 2 | 283 | 0.00 % |
| bytemuck | 5 | 184 | 0.00 % |
| unicode_properties | 2 | 176 | 0.00 % |
| crossbeam_utils | 1 | 84 | 0.00 % |
| serde_arrays | 1 | 21 | 0.00 % |
