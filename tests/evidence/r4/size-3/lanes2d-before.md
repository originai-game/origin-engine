# lanes2d, before (shipping `wasm` profile; scripts/wasm-size-report.sh -n 60, then twiggy 0.8.0 top per crate)

```
lanes2d.wasm: 18678880 B raw, 11154983 B after wasm-opt, 3124590 B brotli -q 11 (74.5 % of wasm_core_brotli_bytes, 4194304 B)
```

Per crate: code bodies and the data section, 11,050,038 B raw; "brotli alone" is each group compressed by itself (they sum to 3,284,360 B, more than the module's brotli)

| Crate | Functions | Raw bytes | Raw share | Brotli alone | Brotli share |
|---|---:|---:|---:|---:|---:|
| og_runtime | 966 | 1,080,176 | 9.78 % | 346,972 | 10.56 % |
| [data] | 1 | 1,412,579 | 12.78 % | 320,346 | 9.75 % |
| og_host_web (exports) | 383 | 1,115,074 | 10.09 % | 317,801 | 9.68 % |
| rapier2d | 365 | 827,572 | 7.49 % | 268,592 | 8.18 % |
| og_render | 628 | 735,690 | 6.66 % | 236,065 | 7.19 % |
| naga | 1,255 | 625,099 | 5.66 % | 185,714 | 5.65 % |
| parry2d | 799 | 461,815 | 4.18 % | 131,304 | 4.00 % |
| wgpu | 384 | 353,449 | 3.20 % | 127,456 | 3.88 % |
| og_physics | 264 | 479,943 | 4.34 % | 123,969 | 3.77 % |
| og_anim | 328 | 354,274 | 3.21 % | 106,628 | 3.25 % |
| og_ui | 256 | 310,194 | 2.81 % | 105,816 | 3.22 % |
| std | 1,614 | 395,194 | 3.58 % | 103,589 | 3.15 % |
| wgpu_core | 951 | 329,076 | 2.98 % | 93,288 | 2.84 % |
| wgpu_hal | 355 | 166,638 | 1.51 % | 61,405 | 1.87 % |
| og_nav | 88 | 148,260 | 1.34 % | 49,940 | 1.52 % |
| og_ecs | 470 | 181,152 | 1.64 % | 49,155 | 1.50 % |
| bincode | 72 | 174,364 | 1.58 % | 49,075 | 1.49 % |
| ttf_parser | 294 | 129,823 | 1.17 % | 42,807 | 1.30 % |
| rustybuzz | 307 | 109,333 | 0.99 % | 40,482 | 1.23 % |
| og_core | 519 | 169,309 | 1.53 % | 35,970 | 1.10 % |
| og_audio | 116 | 84,244 | 0.76 % | 32,563 | 0.99 % |
| [unnamed] | 400 | 102,004 | 0.92 % | 30,146 | 0.92 % |
| og_cook | 288 | 95,097 | 0.86 % | 29,946 | 0.91 % |
| og_ai | 73 | 88,793 | 0.80 % | 29,402 | 0.90 % |
| js_sys | 74 | 75,675 | 0.68 % | 27,550 | 0.84 % |
| serde_json | 254 | 79,993 | 0.72 % | 26,213 | 0.80 % |
| rayon | 36 | 81,457 | 0.74 % | 25,682 | 0.78 % |
| og_probe | 88 | 68,532 | 0.62 % | 25,656 | 0.78 % |
| rayon_core | 278 | 136,759 | 1.24 % | 21,228 | 0.65 % |
| og_host_web | 80 | 46,844 | 0.42 % | 17,281 | 0.53 % |
| og_geom | 60 | 42,945 | 0.39 % | 16,725 | 0.51 % |
| og_lanes | 92 | 49,137 | 0.44 % | 14,831 | 0.45 % |
| og_tilemap | 52 | 29,668 | 0.27 % | 12,221 | 0.37 % |
| og_input | 71 | 48,443 | 0.44 % | 11,987 | 0.36 % |
| glam | 75 | 30,817 | 0.28 % | 11,575 | 0.35 % |
| og_tween | 62 | 28,889 | 0.26 % | 10,838 | 0.33 % |
| og_i18n | 87 | 26,557 | 0.24 % | 10,581 | 0.32 % |
| og_net | 63 | 28,502 | 0.26 % | 10,419 | 0.32 % |
| wgpu_types | 283 | 29,299 | 0.27 % | 10,260 | 0.31 % |
| indexmap | 124 | 42,698 | 0.39 % | 9,903 | 0.30 % |
| miniz_oxide | 30 | 20,037 | 0.18 % | 8,856 | 0.27 % |
| codespan_reporting | 59 | 21,801 | 0.20 % | 8,762 | 0.27 % |
| [other] | 95 | 22,550 | 0.20 % | 8,400 | 0.26 % |
| spade | 43 | 23,650 | 0.21 % | 8,064 | 0.25 % |
| compiler_builtins | 37 | 17,989 | 0.16 % | 7,737 | 0.24 % |
| robust | 7 | 28,597 | 0.26 % | 7,668 | 0.23 % |
| og_timeline | 42 | 12,965 | 0.12 % | 5,637 | 0.17 % |
| libm | 13 | 8,992 | 0.08 % | 3,332 | 0.10 % |
| og_blob | 34 | 10,719 | 0.10 % | 3,324 | 0.10 % |
| naga_types | 57 | 7,048 | 0.06 % | 3,274 | 0.10 % |
| smallvec | 64 | 10,957 | 0.10 % | 3,050 | 0.09 % |
| arrayvec | 98 | 7,046 | 0.06 % | 2,965 | 0.09 % |
| glow | 77 | 9,691 | 0.09 % | 2,659 | 0.08 % |
| crossbeam_channel | 28 | 7,933 | 0.07 % | 2,483 | 0.08 % |
| nalgebra | 9 | 5,464 | 0.05 % | 2,398 | 0.07 % |
| dlmalloc | 22 | 5,813 | 0.05 % | 2,367 | 0.07 % |
| unicode_bidi | 13 | 5,831 | 0.05 % | 2,328 | 0.07 % |
| wasm_bindgen | 58 | 5,210 | 0.05 % | 1,848 | 0.06 % |
| crossbeam_epoch | 27 | 4,196 | 0.04 % | 1,516 | 0.05 % |
| rustc_hash | 27 | 3,545 | 0.03 % | 1,354 | 0.04 % |
| parking_lot | 6 | 3,801 | 0.03 % | 1,287 | 0.04 % |
| crossbeam_deque | 19 | 3,781 | 0.03 % | 1,231 | 0.04 % |
| serde_core | 24 | 3,326 | 0.03 % | 1,231 | 0.04 % |
| zmij | 1 | 2,148 | 0.02 % | 1,136 | 0.03 % |
| parking_lot_core | 11 | 1,729 | 0.02 % | 943 | 0.03 % |
| half | 21 | 1,831 | 0.02 % | 827 | 0.03 % |
| unicode_segmentation | 8 | 1,580 | 0.01 % | 811 | 0.02 % |
| foldhash | 4 | 3,041 | 0.03 % | 652 | 0.02 % |
| once_cell | 5 | 945 | 0.01 % | 545 | 0.02 % |
| glamx | 3 | 1,093 | 0.01 % | 525 | 0.02 % |
| wgpu_naga_bridge | 3 | 1,393 | 0.01 % | 517 | 0.02 % |
| wasm_bindgen_rayon | 3 | 1,084 | 0.01 % | 481 | 0.01 % |
| bitflags | 9 | 1,410 | 0.01 % | 477 | 0.01 % |
| bit_vec | 9 | 717 | 0.01 % | 442 | 0.01 % |
| raw_window_handle | 19 | 1,170 | 0.01 % | 412 | 0.01 % |
| slotmap | 6 | 681 | 0.01 % | 406 | 0.01 % |
| web_sys | 27 | 980 | 0.01 % | 404 | 0.01 % |
| bit_set | 6 | 590 | 0.01 % | 396 | 0.01 % |
| fdeflate | 1 | 826 | 0.01 % | 390 | 0.01 % |
| itoa | 3 | 457 | 0.00 % | 324 | 0.01 % |
| wasm_sync | 4 | 424 | 0.00 % | 252 | 0.01 % |
| __rustc | 5 | 316 | 0.00 % | 233 | 0.01 % |
| lock_api | 2 | 278 | 0.00 % | 227 | 0.01 % |
| log | 3 | 340 | 0.00 % | 204 | 0.01 % |
| simd_adler32 | 1 | 250 | 0.00 % | 177 | 0.01 % |
| bytemuck | 5 | 174 | 0.00 % | 168 | 0.01 % |
| unicode_properties | 2 | 171 | 0.00 % | 158 | 0.00 % |
| crossbeam_utils | 1 | 82 | 0.00 % | 78 | 0.00 % |
| serde_arrays | 1 | 19 | 0.00 % | 23 | 0.00 % |

Top 60 functions by raw bytes

| Raw bytes | Brotli alone | Crate | Function |
|---:|---:|---|---|
| 409,980 | 136,629 | og_render | `<og_render::Renderer>::render` |
| 259,132 | 64,835 | og_host_web (exports) | `og_engine_create multivalue shim` |
| 168,031 | 64,119 | og_runtime | `og_runtime::extract::extract` |
| 159,762 | 57,757 | rapier2d | `<rapier2d::pipeline::physics_pipeline::PhysicsPipeline>::step_inner` |
| 119,106 | 34,331 | bincode | `bincode::internal::deserialize_seed::<core::marker::PhantomData<og_physics::dim2::sim::Sim>, bincode::config::WithOtherTrailing<bincode::config::DefaultOptio...` |
| 110,936 | 36,844 | og_physics | `og_physics::dim2::sim::step_here` |
| 95,623 | 35,728 | wgpu | `<wgpu::backend::wgpu_core::CoreCommandEncoder as wgpu::dispatch::CommandEncoderInterface>::finish` |
| 88,541 | 29,688 | og_runtime | `og_runtime::modules::mount_back::{closure#0}` |
| 68,650 | 24,717 | js_sys | `js_sys::futures::future_to_promise::<og_host_web::canvas::_::__wasm_bindgen_generated_og_web_attach_canvas::{closure#0}>::{closure#0}::{closure#0}` |
| 61,568 | 19,845 | rapier2d | `rapier2d::dynamics::solver::staged_island_solver::worker::run_worker` |
| 54,165 | 21,628 | og_ui | `<og_ui::text::FontMetrics>::run_one` |
| 51,592 | 18,274 | og_anim | `og_anim::system::install::{closure#0}` |
| 51,310 | 17,564 | og_host_web (exports) | `og_render_csg_geometry multivalue shim` |
| 49,841 | 9,241 | og_physics | `og_physics::gather::gather` |
| 41,752 | 16,728 | wgpu_hal | `<wgpu_hal::gles::Device>::create_pipeline` |
| 40,960 | 15,335 | rapier2d | `<rapier2d::pipeline::physics_pipeline::PhysicsPipeline>::detect_collisions` |
| 37,980 | 11,303 | og_anim | `og_anim::rig2d::parse_body` |
| 37,436 | 14,334 | og_runtime | `og_runtime::timeline::decode` |
| 31,474 | 6,518 | og_physics | `<og_physics::PhysicsState as og_ecs::state::StateBlock>::save` |
| 30,888 | 11,858 | naga | `<naga::valid::Validator>::validate_impl` |
| 30,442 | 6,608 | og_input | `<og_input::sink::InputLayer as og_input::sink::InputSink>::attach` |
| 29,989 | 11,074 | og_runtime | `og_runtime::timeline::install::{closure#5}` |
| 29,442 | 10,178 | rapier2d | `rapier2d::dynamics::solver::staged_island_solver::solve::solve_pass` |
| 28,824 | 11,824 | og_runtime | `og_runtime::assets::spawn_scene` |
| 28,428 | 10,045 | og_host_web (exports) | `og_room_deliver` |
| 28,134 | 8,985 | naga | `<naga::front::wgsl::lower::Lowerer>::call` |
| 27,640 | 9,662 | naga | `<naga::valid::Validator>::validate_function` |
| 27,251 | 9,812 | og_runtime | `og_runtime::engine::call_rpc` |
| 26,443 | 11,083 | og_host_web (exports) | `og_anim_retarget multivalue shim` |
| 26,278 | 6,682 | naga | `<naga::back::glsl::writer::Writer<&mut alloc::string::String>>::write_expr` |
| 26,151 | 9,936 | og_runtime | `og_runtime::anim::load_animation` |
| 25,835 | 10,087 | naga | `<naga::front::wgsl::Frontend>::parse` |
| 24,794 | 9,638 | og_audio | `og_audio::system::run` |
| 24,712 | 9,683 | og_host_web (exports) | `og_load_flipbook multivalue shim` |
| 24,464 | 3,865 | rapier2d | `core::slice::sort::unstable::quicksort::quicksort::<(&rapier2d::dynamics::island_manager::persistent::JointLinkKey, &(u32, u32)), <[(&rapier2d::dynamics::isl...` |
| 23,558 | 7,729 | og_runtime | `og_runtime::modules::load_style` |
| 23,500 | 5,170 | naga | `<naga::front::wgsl::error::Error>::as_parse_error` |
| 22,998 | 8,393 | og_host_web (exports) | `og_anim_graph multivalue shim` |
| 22,839 | 9,530 | og_nav | `og_nav::system::run` |
| 22,699 | 8,984 | ttf_parser | `<ttf_parser::Face>::parse` |
| 22,646 | 8,314 | og_anim | `<og_anim::rig2d::Rig2dAsset>::pose_of` |
| 22,355 | 9,447 | wgpu | `<wgpu::backend::wgpu_core::CoreQueue as wgpu::dispatch::QueueInterface>::submit` |
| 22,230 | 8,938 | og_probe | `<og_probe::probe::Probe>::execute` |
| 22,222 | 8,517 | og_render | `og_render::id_pass::render_ids_from` |
| 21,948 | 9,318 | wgpu | `<wgpu::backend::wgpu_core::CoreDevice as wgpu::dispatch::DeviceInterface>::create_render_pipeline` |
| 21,825 | 9,515 | og_runtime | `og_runtime::stream::fetch` |
| 21,794 | 9,105 | rapier2d | `<rapier2d::geometry::narrow_phase::NarrowPhase>::compute_contacts::{closure#3}` |
| 20,220 | 7,857 | og_ui | `<og_ui::ui::Ui>::edit` |
| 19,930 | 8,146 | og_ai | `og_ai::system::install::{closure#0}` |
| 19,889 | 8,477 | og_runtime | `og_runtime::assets::add_gltf` |
| 19,691 | 6,187 | naga | `<naga::valid::Validator>::validate_block_impl` |
| 19,646 | 7,317 | og_nav | `<og_nav::mesh::NavMesh>::build_tile` |
| 19,482 | 8,641 | rayon | `rayon::iter::plumbing::bridge_producer_consumer::helper::<rayon::iter::enumerate::EnumerateProducer<rayon::slice::IterMutProducer<rapier2d::dynamics::solver:...` |
| 19,477 | 6,969 | og_physics | `og_physics::schedule::{closure#0}` |
| 19,292 | 8,353 | og_host_web (exports) | `og_load_rig2d multivalue shim` |
| 18,969 | 7,834 | og_ui | `og_ui::text::bidi_runs` |
| 18,861 | 6,076 | og_cook | `<og_cook::runtime::Bundle>::model` |
| 18,300 | 7,537 | wgpu_hal | `<wgpu_hal::gles::web::Instance as wgpu_hal::dynamic::instance::DynInstance>::enumerate_adapters` |
| 17,828 | 6,367 | wgpu_hal | `<wgpu_hal::gles::Queue as wgpu_hal::dynamic::queue::DynQueue>::submit` |
| 17,506 | 7,311 | rapier2d | `<rapier2d::dynamics::soft_body::soft_body_set::soft_body_set::SoftBodySet>::split_cluster` |

twiggy top, per crate (shallow bytes; names and producers left out): 11,154,713 B

| Crate | Items | Shallow bytes | Share |
|---|---:|---:|---:|
| [data] | 3 | 1,412,578 | 12.66 % |
| og_host_web (exports) | 383 | 1,116,230 | 10.01 % |
| og_runtime | 966 | 1,082,788 | 9.71 % |
| rapier2d | 365 | 828,657 | 7.43 % |
| og_render | 628 | 737,340 | 6.61 % |
| naga | 1255 | 628,118 | 5.63 % |
| og_physics | 264 | 480,660 | 4.31 % |
| parry2d | 799 | 463,920 | 4.16 % |
| std | 1614 | 399,146 | 3.58 % |
| og_anim | 328 | 355,171 | 3.18 % |
| wgpu | 384 | 354,399 | 3.18 % |
| wgpu_core | 951 | 331,445 | 2.97 % |
| og_ui | 256 | 310,918 | 2.79 % |
| og_ecs | 470 | 182,475 | 1.64 % |
| bincode | 72 | 174,573 | 1.57 % |
| og_core | 519 | 170,613 | 1.53 % |
| wgpu_hal | 355 | 167,475 | 1.50 % |
| og_nav | 88 | 148,508 | 1.33 % |
| rayon_core | 278 | 137,552 | 1.23 % |
| ttf_parser | 294 | 130,581 | 1.17 % |
| rustybuzz | 307 | 110,082 | 0.99 % |
| [unnamed] | 400 | 103,054 | 0.92 % |
| og_cook | 288 | 95,754 | 0.86 % |
| og_ai | 73 | 88,993 | 0.80 % |
| og_audio | 116 | 84,544 | 0.76 % |
| rayon | 36 | 81,563 | 0.73 % |
| serde_json | 254 | 80,638 | 0.72 % |
| js_sys | 74 | 75,837 | 0.68 % |
| og_probe | 88 | 68,768 | 0.62 % |
| [wasm structure] | 1508 | 68,683 | 0.62 % |
| og_lanes | 92 | 49,401 | 0.44 % |
| og_input | 71 | 48,601 | 0.44 % |
| og_host_web | 80 | 47,053 | 0.42 % |
| og_geom | 60 | 43,124 | 0.39 % |
| indexmap | 124 | 42,995 | 0.39 % |
| glam | 75 | 31,015 | 0.28 % |
| wgpu_types | 283 | 29,916 | 0.27 % |
| og_tilemap | 52 | 29,800 | 0.27 % |
| og_tween | 62 | 29,053 | 0.26 % |
| og_net | 63 | 28,669 | 0.26 % |
| robust | 7 | 28,622 | 0.26 % |
| og_i18n | 87 | 26,777 | 0.24 % |
| spade | 43 | 23,763 | 0.21 % |
| [other] | 100 | 22,814 | 0.20 % |
| codespan_reporting | 59 | 21,948 | 0.20 % |
| miniz_oxide | 30 | 20,109 | 0.18 % |
| compiler_builtins | 37 | 18,101 | 0.16 % |
| og_timeline | 42 | 13,075 | 0.12 % |
| smallvec | 64 | 11,110 | 0.10 % |
| og_blob | 34 | 10,804 | 0.10 % |
| glow | 77 | 9,879 | 0.09 % |
| libm | 13 | 9,030 | 0.08 % |
| crossbeam_channel | 28 | 8,004 | 0.07 % |
| arrayvec | 98 | 7,254 | 0.07 % |
| naga_types | 57 | 7,174 | 0.06 % |
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
| fdeflate | 1 | 829 | 0.01 % |
| bit_vec | 9 | 736 | 0.01 % |
| slotmap | 6 | 695 | 0.01 % |
| bit_set | 6 | 604 | 0.01 % |
| itoa | 3 | 465 | 0.00 % |
| wasm_sync | 4 | 433 | 0.00 % |
| log | 3 | 348 | 0.00 % |
| __rustc | 5 | 327 | 0.00 % |
| lock_api | 2 | 283 | 0.00 % |
| simd_adler32 | 1 | 253 | 0.00 % |
| bytemuck | 5 | 184 | 0.00 % |
| unicode_properties | 2 | 176 | 0.00 % |
| crossbeam_utils | 1 | 84 | 0.00 % |
| serde_arrays | 1 | 21 | 0.00 % |
